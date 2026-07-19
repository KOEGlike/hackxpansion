use embassy_rp::{
    gpio::{Level, Output},
    spi,
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use ti_adc_expander::{Ads7028, AnalogIn};

use xpanse_api::{
    bus::allocator::BusAllocator,
    bus::spi::SpiBusHandle,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::Registry,
};

pub struct SpiAdcDriver;

type AdcSpiDevice = ExclusiveDevice<SpiBusHandle, Output<'static>, Delay>;
type ConfiguredAdc = Ads7028<
    AdcSpiDevice,
    AnalogIn,
    AnalogIn,
    AnalogIn,
    AnalogIn,
    AnalogIn,
    AnalogIn,
    AnalogIn,
    AnalogIn,
>;

pub struct SpiAdc {
    adc: ConfiguredAdc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SpiAdcError {
    InvalidChannel,
    Bus,
}

impl SpiAdc {
    pub async fn read_channel(&mut self, channel: u8) -> Result<u16, SpiAdcError> {
        match channel {
            0 => self.adc.read_ch0().await.map_err(|_| SpiAdcError::Bus),
            1 => self.adc.read_ch1().await.map_err(|_| SpiAdcError::Bus),
            2 => self.adc.read_ch2().await.map_err(|_| SpiAdcError::Bus),
            3 => self.adc.read_ch3().await.map_err(|_| SpiAdcError::Bus),
            4 => self.adc.read_ch4().await.map_err(|_| SpiAdcError::Bus),
            5 => self.adc.read_ch5().await.map_err(|_| SpiAdcError::Bus),
            6 => self.adc.read_ch6().await.map_err(|_| SpiAdcError::Bus),
            7 => self.adc.read_ch7().await.map_err(|_| SpiAdcError::Bus),
            _ => Err(SpiAdcError::InvalidChannel),
        }
    }
}

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
        let mut config = spi::Config::default();
        config.frequency = 400_000;
        let spi = bus_allocator
            .create_spi_bitbang::<G::SPI>(gpio_bank.gpio2, gpio_bank.gpio4, gpio_bank.gpio3, config)
            .map_err(|_| DriverError::InitFailed)?;

        let chip_select = Output::new(gpio_bank.gpio9.into(), Level::High);
        let device =
            ExclusiveDevice::new(spi, chip_select, Delay).map_err(|_| DriverError::InitFailed)?;

        let mut adc = Ads7028::new(device);

        adc.clear_bor().await.map_err(|_| DriverError::InitFailed)?;
        adc.set_oversampling(ti_adc_expander::OversamplingRatio::Osr16)
            .await
            .map_err(|_| DriverError::InitFailed)?;

        let adc = adc
            .configure_ch0_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch1_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch2_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch3_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch4_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch5_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch6_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?
            .configure_ch7_as_analog()
            .await
            .map_err(|_| DriverError::InitFailed)?;

        registry.register(slot, SpiAdcDriver::ID, SpiAdc { adc });

        Ok(())
    }
}
