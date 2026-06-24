#![no_std]

extern crate alloc;

pub mod spi_adc;

use alloc::sync::Arc;

use embassy_executor::SendSpawner;
use embassy_rp::{
    Peri,
    gpio::{AnyPin, Level, Output},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

use xpanse_driver_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{pin_button, A},
    metadata::{ModuleID, ModuleDetectResistor, ModuleSlot},
    registry::Registry,
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
        _bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        let spawner = SendSpawner::for_current_executor().await;
        let pressed = Arc::new(Signal::new());

        spawner
            .spawn(blink_led(gpio_bank.gpio0.into(), pressed.clone()).unwrap());

        registry.register(slot, TestDriver::ID, pin_button::<A>(gpio_bank.gpio3.into()));

        Ok(())
    }
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
