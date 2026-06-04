#![no_std]
#![no_main]

extern crate alloc;

use defmt::*;
use embassy_executor::{Executor, SendSpawner, Spawner};
use embassy_rp::{
    Peri,
    gpio::{self, AnyPin, Level, Output},
    i2c,
    multicore::{Stack, spawn_core1},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;
use static_cell::StaticCell;
use xpanse::{
    adc::init_adc, core0::core0_task, core1::core1_task, display::init_display, resource_split::*,
    split_resources,
};
use xpanse_driver_api::{
    driver::Driver,
    gpio_bank::{BankPins, GpioBank},
};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Hackxpansion"),
    embassy_rp::binary_info::rp_program_description!(c"Firmware for Hackxpansion"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(unwrap!(core1_task(
                    r.gpio_bank_0,
                    r.gpio_bank_1,
                    r.gpio_bank_2,
                    r.gpio_bank_3,
                    r.i2c_pins,
                    r.remaining_peris
                )))
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(unwrap!(core0_task(r.display))));
}

struct TestDriver<G>
where
    G: BankPins,
{
    btn_pin: Peri<'static, G::GPIO3>,
    pressed: Signal<ThreadModeRawMutex, bool>,
}

impl<G: BankPins> Driver<G> for TestDriver<G> {
    async fn init(&mut self, gpio_bank: GpioBank<G>) -> Self {
        let spawner = SendSpawner::for_current_executor().await;
        let pressed = Signal::new();
        spawner.spawn(blink_led(gpio_bank.gpio0.into(), pressed).unwrap());
        Self {
            btn_pin: gpio_bank.gpio3,
            pressed,
        }
    }
}

#[embassy_executor::task]
async fn blink_led(pin: Peri<'static, AnyPin>, pressed: Signal<ThreadModeRawMutex, bool>) {
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
