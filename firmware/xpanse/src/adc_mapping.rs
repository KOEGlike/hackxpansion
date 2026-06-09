use embassy_rp::i2c;
use libm::pow;
use xpanse_driver_api::metadata::{AVDD, ModuleDetectResistor, ModuleID, ModuleSlot};

use crate::adc;

pub async fn map_adc<'a>(adc: &mut adc::Adc<'a>, slot: ModuleSlot) -> Result<ModuleID, i2c::Error> {
    let (md0_reading, md1_reading) = match slot {
        ModuleSlot::FrontRight => (adc.read_ch2().await?, adc.read_ch3().await?),
        ModuleSlot::FrontLeft => (adc.read_ch0().await?, adc.read_ch1().await?),
        ModuleSlot::BackRight => (adc.read_ch4().await?, adc.read_ch5().await?),
        ModuleSlot::BackLeft => (adc.read_ch6().await?, adc.read_ch7().await?),
    };

    let md0_volatage = AVDD / pow(2.0, 16.0) * md0_reading as f64;
    let md1_voltage = AVDD / pow(2.0, 16.0) * md1_reading as f64;

    let md0 = ModuleDetectResistor::from_voltage(md0_volatage);
    let md1 = ModuleDetectResistor::from_voltage(md1_voltage);
    Ok(ModuleID { md0, md1 })
}
