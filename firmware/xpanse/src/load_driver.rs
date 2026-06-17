#[macro_export]
macro_rules! load_driver {
    ($id:expr, $bank:expr, $slot:expr, $registry:expr, $bus_allocator:expr) => {
        if $id
            == $crate::load_driver::get_driver_id::<test_driver::TestDriver, _, _>(
                &$bank, $registry,
            )
        {
            test_driver::TestDriver::new($bank, $slot, $registry, $bus_allocator).await;
        } else {
            defmt::warn!("Unknown driver ID: {:#?}", $id);
        }
    };
}

pub fn get_driver_id<D, G, R>(
    _bank: &xpanse_driver_api::gpio_bank::GpioBank<G>,
    _registry: &mut R,
) -> xpanse_driver_api::metadata::ModuleID
where
    G: xpanse_driver_api::gpio_bank::BankPins,
    D: xpanse_driver_api::driver::Driver<G, R>,
{
    D::ID
}
