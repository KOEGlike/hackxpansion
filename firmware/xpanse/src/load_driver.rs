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
            match test_driver::TestDriver::create(bank, slot, registry, bus).await {
                Ok(()) => defmt::info!("Test driver initialized in {:?}", slot),
                Err(error) => {
                    defmt::error!("Test driver init failed in {:?}: {:?}", slot, error)
                }
            }
        }
        Some(id) if id == test_driver::spi_adc::SpiAdcDriver::ID => {
            match test_driver::spi_adc::SpiAdcDriver::create(bank, slot, registry, bus).await {
                Ok(()) => defmt::info!("SPI ADC driver initialized in {:?}", slot),
                Err(error) => {
                    defmt::error!("SPI ADC driver init failed in {:?}: {:?}", slot, error)
                }
            }
        }
        Some(id) => defmt::warn!("unknown driver id {:?} in {:?}", id, slot),
        None => defmt::info!("no driver to load in {:?}", slot),
    }
}
