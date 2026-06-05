use crate::gpio_bank::{BankPins, GpioBank};
use crate::metadata::Slots;
use core::future::Future;

pub trait Driver<G: BankPins, R> {
    fn new(gpio_bank: GpioBank<G>, slot: Slots, registry: &mut R) -> impl Future<Output = Self>;
}
