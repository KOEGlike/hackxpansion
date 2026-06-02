use embedded_hal_async::i2c::I2c;
use ti_adc_expander::{Address, Ads7128, OversamplingRatio};

type Adc<I> = ti_adc_expander::Driver<
    ti_adc_expander::I2cInterface<I>,
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

pub async fn init_adc<I: I2c>(i2c: I) -> Result<Adc<I>, I::Error> {
    let mut adc = Ads7128::new(i2c, Address::X10);
    adc.clear_bor().await?;
    adc.set_oversampling(OversamplingRatio::Osr16).await?;

    let adc = adc
        .configure_ch0_as_analog()
        .await?
        .configure_ch1_as_analog()
        .await?
        .configure_ch2_as_analog()
        .await?
        .configure_ch3_as_analog()
        .await?
        .configure_ch4_as_analog()
        .await?
        .configure_ch5_as_analog()
        .await?
        .configure_ch6_as_analog()
        .await?
        .configure_ch7_as_analog()
        .await?;

    Ok(adc)
}
