use crate::gpio_bank::{BankPins, GpioBank};
use crate::metadata::{ModuleID, ModuleSlot};
use core::future::Future;

pub trait Driver<G: BankPins, R> {
    const ID: ModuleID;

    fn new(gpio_bank: GpioBank<G>, slot: ModuleSlot, registry: &mut R) -> impl Future<Output = ()>;
}
