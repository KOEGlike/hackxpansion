use libm::pow;
use xpanse_api::bus::i2c::I2cError;
use xpanse_api::metadata::{AVDD, ModuleDetectResistor, ModuleID, ModuleSlot};

use crate::adc;

pub async fn map_adc<'a>(
    adc: &mut adc::Adc<'a>,
    slot: ModuleSlot,
) -> Result<Option<ModuleID>, I2cError> {
    let (md0_reading, md1_reading) = match slot {
        ModuleSlot::FrontRight => (adc.read_ch2().await?, adc.read_ch3().await?),
        ModuleSlot::FrontLeft => (adc.read_ch0().await?, adc.read_ch1().await?),
        ModuleSlot::BackRight => (adc.read_ch4().await?, adc.read_ch5().await?),
        ModuleSlot::BackLeft => (adc.read_ch6().await?, adc.read_ch7().await?),
    };

    let md0_volatage = AVDD / pow(2.0, 16.0) * md0_reading as f64;
    let md1_voltage = AVDD / pow(2.0, 16.0) * md1_reading as f64;

    let Some(md0) = ModuleDetectResistor::from_voltage(md0_volatage) else {
        return Ok(None);
    };
    let Some(md1) = ModuleDetectResistor::from_voltage(md1_voltage) else {
        return Ok(None);
    };
    Ok(Some(ModuleID { md0, md1 }))
}
