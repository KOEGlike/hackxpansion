use crate::gpio_bank::{BankPins, GpioBank};
use core::future::Future;

pub trait Driver<'a, G: BankPins> {
    fn init(&mut self, gpio_bank: GpioBank<'a, G>) -> impl Future<Output = ()>;
}
