use crate::gpio_bank::{BankPins, GpioBank};
use core::future::Future;

pub trait Driver<G: BankPins> {
    fn new(gpio_bank: GpioBank<G>) -> impl Future<Output = Self>;
}
