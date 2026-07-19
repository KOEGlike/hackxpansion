use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    metadata::{ModuleID, ModuleSlot},
    registry::Registry,
};

pub async fn load_driver<G: BankPins>(
    id: Option<ModuleID>,
    bank: GpioBank<G>,
    slot: ModuleSlot,
    registry: &mut Registry,
    bus: &mut BusAllocator,
) {
    match id {
        Some(id) if id == test_driver::TestDriver::ID => {
            if let Err(e) = test_driver::TestDriver::create(bank, slot, registry, bus).await {
                defmt::error!("Test driver init failed: {:?}", e);
            }
        }
        Some(id) if id == test_driver::spi_adc::SpiAdcDriver::ID => {
            if let Err(e) =
                test_driver::spi_adc::SpiAdcDriver::create(bank, slot, registry, bus).await
            {
                defmt::error!("SPI ADC driver init failed: {:?}", e);
            }
        }
        Some(id) => defmt::warn!("unknown driver id: {:?}", id),
        None => defmt::debug!("no module detected in {:?}", slot),
    }
}
