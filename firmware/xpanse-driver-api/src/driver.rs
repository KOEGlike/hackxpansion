use crate::bus::allocator::BusAllocator;
use crate::gpio_bank::{BankPins, GpioBank};
use crate::metadata::{ModuleID, ModuleSlot};
use crate::registry::Registry;
use core::future::Future;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum DriverError {
    InitFailed,
}

pub trait DriverMeta {
    const ID: ModuleID;
}

pub trait Driver<G: BankPins>: DriverMeta {
    fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> impl Future<Output = Result<(), DriverError>>;
}
