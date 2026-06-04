use embassy_rp::{gpio, i2c};
use xpanse_driver_api::gpio_bank::GpioBank;

use crate::{adc::init_adc, resource_split::*};

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

#[embassy_executor::task]
pub async fn core1_task(
    gpio_bank_0: GpioBankPeris0,
    gpio_bank_1: GpioBankPeris1,
    gpio_bank_2: GpioBankPeris2,
    gpio_bank_3: GpioBankPeris3,
    mut i2c_pins: I2cPinPeris,
    mut remaining_peris: RemainingPeris,
) {
    let gpio_bank_0 = GpioBank::new(
        gpio_bank_0.gpio0,      //0
        gpio_bank_0.gpio1,      //1
        gpio_bank_0.gpio2,      //2
        gpio_bank_0.gpio3,      //3
        gpio_bank_0.gpio4,      //4
        gpio_bank_0.gpio5,      //5
        gpio_bank_0.gpio6,      //6
        gpio_bank_0.gpio7,      //7
        gpio_bank_0.gpio8,      //8
        gpio_bank_0.gpio9,      //9
        gpio_bank_0.pwm_slice1, //10
        gpio_bank_0.pwm_slice2, //11
        gpio_bank_0.pwm_slice3, //12
    );

    let gpio_bank_1 = GpioBank::new(
        gpio_bank_1.gpio0,      //0
        gpio_bank_1.gpio1,      //1
        gpio_bank_1.gpio2,      //2
        gpio_bank_1.gpio3,      //3
        gpio_bank_1.gpio4,      //4
        gpio_bank_1.gpio5,      //5
        gpio_bank_1.gpio6,      //6
        gpio_bank_1.gpio7,      //7
        gpio_bank_1.gpio8,      //8
        gpio_bank_1.gpio9,      //9
        gpio_bank_1.pwm_slice1, //10
        gpio_bank_1.pwm_slice2, //11
        gpio_bank_1.pwm_slice3, //12
    );

    let gpio_bank_2 = GpioBank::new(
        gpio_bank_2.gpio0,      //0
        gpio_bank_2.gpio1,      //1
        gpio_bank_2.gpio2,      //2
        gpio_bank_2.gpio3,      //3
        gpio_bank_2.gpio4,      //4
        gpio_bank_2.gpio5,      //5
        gpio_bank_2.gpio6,      //6
        gpio_bank_2.gpio7,      //7
        gpio_bank_2.gpio8,      //8
        gpio_bank_2.gpio9,      //9
        gpio_bank_2.pwm_slice1, //10
        gpio_bank_2.pwm_slice2, //11
        gpio_bank_2.pwm_slice3, //12
    );

    let gpio_bank_3 = GpioBank::new(
        gpio_bank_3.gpio0,      //0
        gpio_bank_3.gpio1,      //1
        gpio_bank_3.gpio2,      //2
        gpio_bank_3.gpio3,      //3
        gpio_bank_3.gpio4,      //4
        gpio_bank_3.gpio5,      //5
        gpio_bank_3.gpio6,      //6
        gpio_bank_3.gpio7,      //7
        gpio_bank_3.gpio8,      //8
        gpio_bank_3.gpio9,      //9
        gpio_bank_3.pwm_slice1, //10
        gpio_bank_3.pwm_slice2, //11
        gpio_bank_3.pwm_slice3, //12
    );

    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(
        remaining_peris.i2c1.reborrow(),
        i2c_pins.scl.reborrow(),
        i2c_pins.sda.reborrow(),
        Irqs,
        config,
    );

    let mut adc = init_adc(bus).await.unwrap();

    let raw = match adc.read_ch0_polled().await {
        Ok(raw) => raw,
        Err(e) => {
            defmt::error!("Error while reading adc ch0 {}", e);
            return;
        }
    };
}
