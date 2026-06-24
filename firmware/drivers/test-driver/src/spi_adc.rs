use embassy_rp::spi;
use ti_adc_expander::Ads7028;

use xpanse_driver_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    metadata::{ModuleID, ModuleDetectResistor, ModuleSlot},
    registry::Registry,
};

pub struct SpiAdcDriver;

impl DriverMeta for SpiAdcDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R4K7,
        md1: ModuleDetectResistor::R10K,
    };
}

impl<G: BankPins> Driver<G> for SpiAdcDriver {
    async fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        let spi = bus_allocator.create_spi_bitbang::<G::SPI>(
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

        registry.register(slot, SpiAdcDriver::ID, SpiAdcDriver);

        Ok(())
    }
}
