use embassy_rp::{Peri, i2c, peripherals::I2C1};
use ti_adc_expander::{Address, Ads7128, OversamplingRatio};

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

pub type Adc<'a> = ti_adc_expander::Driver<
    ti_adc_expander::I2cInterface<embassy_rp::i2c::I2c<'a, I2C1, embassy_rp::i2c::Async>>,
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

pub async fn init_adc<'a>(
    i2c: Peri<'a, I2C1>,
    sda: Peri<'a, impl i2c::SdaPin<I2C1>>,
    scl: Peri<'a, impl i2c::SclPin<I2C1>>,
) -> Result<Adc<'a>, i2c::Error> {
    let config = embassy_rp::i2c::Config::default();
    let i2c = embassy_rp::i2c::I2c::new_async(i2c, scl, sda, Irqs, config);

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
