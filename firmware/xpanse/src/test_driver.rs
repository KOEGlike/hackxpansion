extern crate alloc;

use embassy_executor::SendSpawner;
use embassy_rp::{
    Peri,
    gpio::{AnyPin, Level, Output},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use xpanse_driver_api::{
    driver::Driver,
    gpio_bank::{BankPins, GpioBank},
};

use alloc::sync::Arc;

struct TestDriver<G>
where
    G: BankPins,
{
    btn_pin: Peri<'static, G::GPIO3>,
    pressed: Arc<Signal<ThreadModeRawMutex, bool>>,
}

impl<G: BankPins> Driver<G> for TestDriver<G> {
    async fn init(gpio_bank: GpioBank<G>) -> Self {
        let spawner = SendSpawner::for_current_executor().await;
        let pressed = Arc::new(Signal::new());
        spawner.spawn(blink_led(gpio_bank.gpio0.into(), pressed.clone()).unwrap());
        Self {
            btn_pin: gpio_bank.gpio3,
            pressed,
        }
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
