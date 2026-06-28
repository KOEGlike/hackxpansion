use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    metadata::{ModuleID, ModuleSlot},
    registry::Registry,
};

pub async fn load_driver<G: BankPins>(
    id: ModuleID,
    bank: GpioBank<G>,
    slot: ModuleSlot,
    registry: &mut Registry,
    bus: &mut BusAllocator,
) {
    match id {
        id if id == test_driver::TestDriver::ID => {
            if let Err(e) = test_driver::TestDriver::create(bank, slot, registry, bus).await {
                defmt::error!("driver init failed: {:?}", e);
            }
        }
        _ => defmt::warn!("unknown driver id: {:?}", id),
    }
}
