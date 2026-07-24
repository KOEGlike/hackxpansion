use embassy_rp::{Peri, gpio::Pin};
use ti_adc_expander::{Address, Ads7128, OversamplingRatio};
use xpanse_api::bus::{i2c::I2cError, i2c_bitbang::BitBangI2cBus};

pub type Adc<'a> = ti_adc_expander::Driver<
    ti_adc_expander::I2cInterface<BitBangI2cBus<'a>>,
    ti_adc_expander::Ads7x28,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
    ti_adc_expander::AnalogIn,
>;

#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum InitError {
    I2c(I2cError),
}

pub async fn init_adc<'a>(
    sda: Peri<'a, impl Pin>,
    scl: Peri<'a, impl Pin>,
) -> Result<Adc<'a>, InitError> {
    let i2c = BitBangI2cBus::new(scl, sda, 100_000).map_err(InitError::I2c)?;

    let mut adc = Ads7128::new(i2c, Address::X10);
    adc.clear_bor().await.map_err(InitError::I2c)?;
    adc.set_oversampling(OversamplingRatio::Osr16)
        .await
        .map_err(InitError::I2c)?;

    adc.configure_ch0_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch1_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch2_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch3_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch4_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch5_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch6_as_analog()
        .await
        .map_err(InitError::I2c)?
        .configure_ch7_as_analog()
        .await
        .map_err(InitError::I2c)
}
