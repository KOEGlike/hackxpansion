#![no_std]

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{A, B, pin_button},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::Registry,
};

pub struct TwoButtonDriver;

impl DriverMeta for TwoButtonDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R1K6,
        md1: ModuleDetectResistor::R1K5,
    };
}

impl<G: BankPins> Driver<G> for TwoButtonDriver {
    async fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        registry.register(
            slot,
            TwoButtonDriver::ID,
            pin_button::<A>(gpio_bank.gpio0.into()),
        );

        registry.register(
            slot,
            TwoButtonDriver::ID,
            pin_button::<B>(gpio_bank.gpio1.into()),
        );

        let _ = bus_allocator;

        Ok(())
    }
}
