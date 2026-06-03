use crate::{
    driver::{self, Driver},
    gpio_bank::{BankPins, GpioBank},
};

pub trait App {
    fn run(&mut self) -> impl Future<Output = ()>;
    fn new<G1: BankPins, G2: BankPins, G3: BankPins, G4: BankPins>(
        driver_1: impl Driver<G1>,
        driver_2: impl Driver<G2>,
        driver_3: impl Driver<G3>,
        driver_4: impl Driver<G4>,
    ) -> impl Future<Output = Self>;
}
