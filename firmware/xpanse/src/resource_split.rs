use assign_resources::assign_resources;
use embassy_rp::{Peri, peripherals};

assign_resources! {
    gpio_bank_0: GpioBankPeris0{
        gpio0: PIN_37, //0
        gpio1: PIN_36, //1
        gpio2: PIN_30, //2
        gpio3: PIN_24, //3
        gpio4: PIN_27, //4
        gpio5: PIN_8,  //5
        gpio6: PIN_9,  //6
        gpio7: PIN_46, //7
        gpio8: PIN_47, //8
        gpio9: PIN_39, //9
        pwm_slice1:PWM_SLICE11,
        pwm_slice2:PWM_SLICE4,
        pwm_slice3:PWM_SLICE7,
    },
    gpio_bank_1: GpioBankPeris1{
        gpio0: PIN_19, //0
        gpio1: PIN_18, //1
        gpio2: PIN_22, //2
        gpio3: PIN_32, //3
        gpio4: PIN_23, //4
        gpio5: PIN_28, //5
        gpio6: PIN_29, //6
        gpio7: PIN_44, //7
        gpio8: PIN_45, //8
        gpio9: PIN_26, //9
        pwm_slice1: PWM_SLICE10,
        pwm_slice2: PWM_SLICE6,
        pwm_slice3: PWM_SLICE3,
    },
    gpio_bank_2: GpioBankPeris2{
        gpio0: PIN_7,  //0
        gpio1: PIN_6,  //1
        gpio2: PIN_2,  //2
        gpio3: PIN_4,  //3
        gpio4: PIN_3,  //4
        gpio5: PIN_0,  //5
        gpio6: PIN_1,  //6
        gpio7: PIN_40, //7
        gpio8: PIN_41, //8
        gpio9: PIN_33, //9
        pwm_slice1: PWM_SLICE8,
        pwm_slice2: PWM_SLICE0,
        pwm_slice3: PWM_SLICE1,
    },
    gpio_bank_3: GpioBankPeris3{
        gpio0: PIN_13, //0
        gpio1: PIN_16, //1
        gpio2: PIN_10, //2
        gpio3: PIN_12, //3
        gpio4: PIN_11, //4
        gpio5: PIN_20, //5
        gpio6: PIN_21, //6
        gpio7: PIN_42, //7
        gpio8: PIN_43, //8
        gpio9: PIN_17, //9
        pwm_slice1: PWM_SLICE9,
        pwm_slice2: PWM_SLICE2,
        pwm_slice3: PWM_SLICE5,
    },
    display: DisplayPeris{
        spi: SPI0,
        clk: PIN_34,
        mosi: PIN_35,
        rst: PIN_5,
        cs: PIN_38,
        dc: PIN_25,
        backlight:PIN_31
    },
    i2c_pins: I2cPinPeris{
        scl: PIN_15,
        sda: PIN_14,
    },
    remaining_peris: RemainingPeris{
        i2c0: I2C0,
        i2c1: I2C1,
        spi1: SPI1,
        uart0: UART0,
        uart1: UART1,
        pio0: PIO0,
        pio1: PIO1,
        pio2: PIO2,
        adc: ADC,
        flash: FLASH,
    },
}
