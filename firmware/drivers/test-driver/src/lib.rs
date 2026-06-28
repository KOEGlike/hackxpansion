#![no_std]

extern crate alloc;

pub mod spi_adc;

use alloc::sync::Arc;

use embassy_executor::SendSpawner;
use embassy_rp::{
    Peri,
    gpio::{AnyPin, Level, Output},
    pio::{Common, Config, Direction, Instance, PioPin, StateMachine},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{A, pin_button},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::Registry,
    with_pio,
};

pub struct TestDriver;

impl DriverMeta for TestDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R1K,
        md1: ModuleDetectResistor::R1K1,
    };
}

impl<G: BankPins> Driver<G> for TestDriver {
    async fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        let spawner = SendSpawner::for_current_executor().await;
        let pressed = Arc::new(Signal::new());

        spawner.spawn(blink_led(gpio_bank.gpio0.into(), pressed.clone()).unwrap());

        registry.register(
            slot,
            TestDriver::ID,
            pin_button::<A>(gpio_bank.gpio3.into()),
        );

        if let Some(pio_access) = bus_allocator.request_pio() {
            with_pio!(
                pio_access,
                common,
                sm,
                start_pio_blink(common, sm, gpio_bank.gpio9)
            );
        } else {
            defmt::warn!("TestDriver: no free PIO state machine for blinky");
        }

        Ok(())
    }
}

fn start_pio_blink<PIO: Instance, const N: usize, P: PioPin>(
    common: &mut Common<'static, PIO>,
    mut sm: StateMachine<'static, PIO, N>,
    pin: Peri<'static, P>,
) {
    let program = pio::pio_asm!("set pins, 1", "set pins, 0",);
    let loaded = common.load_program(&program.program);

    let pin = common.make_pio_pin(pin);
    sm.set_pin_dirs(Direction::Out, &[&pin]);

    let mut cfg = Config::default();
    cfg.set_set_pins(&[&pin]);
    cfg.use_program(&loaded, &[]);
    cfg.clock_divider = fixed::FixedU32::from_num(65536u32);
    sm.set_config(&cfg);

    sm.set_enable(true);
}

#[embassy_executor::task]
async fn blink_led(pin: Peri<'static, AnyPin>, pressed: Arc<Signal<ThreadModeRawMutex, bool>>) {
    let mut led = Output::new(pin, Level::Low);
    loop {
        let val = pressed.wait().await;
        if val {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
