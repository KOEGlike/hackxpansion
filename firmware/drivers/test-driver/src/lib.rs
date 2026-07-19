#![no_std]

pub mod spi_adc;

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{A, pin_button},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::Registry,
};

pub struct TestDriver;

impl DriverMeta for TestDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R1K,
        md1: ModuleDetectResistor::R1K1,
    };
}

impl<G: BankPins> Driver<G> for TestDriver {
    async fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        registry.register(
            slot,
            TestDriver::ID,
            pin_button::<A>(gpio_bank.gpio3.into()),
        );

        let _ = bus_allocator;

        Ok(())
    }
}
