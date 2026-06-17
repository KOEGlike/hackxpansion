use embassy_rp::spi;
use ti_adc_expander::Ads7028;

use xpanse_driver_api::{
    bus::allocator::BusAllocator,
    driver::Driver,
    gpio_bank::{BankPins, GpioBank},
};

pub struct SpiAdcDriver;

impl<G: BankPins, R> Driver<G, R> for SpiAdcDriver
where
    R: xpanse_driver_api::registry::Register<SpiAdcDriver>,
{
    const ID: xpanse_driver_api::metadata::ModuleID = xpanse_driver_api::metadata::ModuleID {
        md0: xpanse_driver_api::metadata::ModuleDetectResistor::R4K7,
        md1: xpanse_driver_api::metadata::ModuleDetectResistor::R10K,
    };

    async fn new(
        gpio_bank: GpioBank<G>,
        slot: xpanse_driver_api::metadata::ModuleSlot,
        registry: &mut R,
        bus_allocator: &mut BusAllocator,
    ) {
        let spi = bus_allocator.create_spi_bus::<G::SPI>(
            gpio_bank.gpio2,
            gpio_bank.gpio4,
            gpio_bank.gpio3,
            spi::Config::default(),
        );

        let mut adc = Ads7028::new(spi);

        adc.clear_bor().await.ok();
        adc.set_oversampling(ti_adc_expander::OversamplingRatio::Osr16)
            .await
            .ok();

        let _adc = adc
            .configure_ch0_as_analog().await.ok().unwrap()
            .configure_ch1_as_analog().await.ok().unwrap()
            .configure_ch2_as_analog().await.ok().unwrap()
            .configure_ch3_as_analog().await.ok().unwrap()
            .configure_ch4_as_analog().await.ok().unwrap()
            .configure_ch5_as_analog().await.ok().unwrap()
            .configure_ch6_as_analog().await.ok().unwrap()
            .configure_ch7_as_analog().await.ok().unwrap();

        registry.register(slot, <Self as Driver<G, R>>::ID, SpiAdcDriver, ());
    }
}

impl xpanse_driver_api::registry::RegisteredResourceInner for SpiAdcDriver {
    type Info = ();
}
