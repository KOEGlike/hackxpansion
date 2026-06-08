#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use alloc::sync::Arc;
use embassy_executor::SendSpawner;
use embassy_rp::{
    Peri,
    gpio::{AnyPin, Input, Level, Output, Pull},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

use xpanse_driver_api::{
    driver::Driver,
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{Button, ButtonA, ButtonB},
};

pub struct SingleButton {
    pin: Input<'static>,
}

impl SingleButton {
    pub fn new(pin: Peri<'static, AnyPin>) -> Self {
        Self {
            pin: Input::new(pin, Pull::Up),
        }
    }
}

impl Button for SingleButton {
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            // Mock waiting for press using embassy_time.
            // In a real hardware driver, you'd use ExtiInput::wait_for_low().
            self.pin.wait_for_low().await;
        })
    }
}

// The primary module driver
pub struct TestDriver {}

impl<G: BankPins, R> Driver<G, R> for TestDriver
where
    R: xpanse_driver_api::registry::Register<dyn Button>
        + xpanse_driver_api::registry::Register<TestDriver>,
{
    const ID: xpanse_driver_api::metadata::ModuleID = xpanse_driver_api::metadata::ModuleID {
        md0: xpanse_driver_api::metadata::ModuleDetectResistor::R1K,
        md1: xpanse_driver_api::metadata::ModuleDetectResistor::R1K1,
    };

    async fn new(
        gpio_bank: GpioBank<G>,
        slot: xpanse_driver_api::metadata::ModuleSlot,
        registry: &mut R,
    ) {
        let spawner = SendSpawner::for_current_executor().await;
        let pressed = Arc::new(Signal::new());

        // Keep the blinking LED from the original code just to show spawning works
        spawner.spawn(blink_led(gpio_bank.gpio0.into(), pressed.clone()).unwrap());

        let button = SingleButton::new(gpio_bank.gpio3.into());
        // We cast to Box<dyn Button> to satisfy the Register trait
        registry.register(
            slot,
            <Self as Driver<G, R>>::ID,
            Box::new(button) as Box<dyn Button>,
        );

        let x = Self {};

        registry.register(slot, <Self as Driver<G, R>>::ID, Box::new(x));
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
