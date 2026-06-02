use core::{
    marker::PhantomData,
    panic::{RefUnwindSafe, UnwindSafe},
};

use embassy_rp::{
    Peri, PeripheralType,
    adc::{AdcChannel, AdcPin},
    gpio::Pin,
    i2c,
    pio::PioPin,
    pwm::{self, ChannelAPin, ChannelBPin},
    spi, uart,
};

pub trait BasePin:
    RefUnwindSafe + Send + Sync + Unpin + UnwindSafe + Copy + Pin + PioPin + PeripheralType
{
}

// Implement BasePin for any type T that implements all these traits
impl<T> BasePin for T where
    T: RefUnwindSafe + Send + Sync + Unpin + UnwindSafe + Copy + Pin + PioPin + PeripheralType
{
}

pub struct GpioBank<
    'a,
    I2C,
    SPI,
    UART,
    PWM_SLICE0,
    PWM_SLICE1,
    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,
    GPIO5,
    GPIO6,
    GPIO7,
    GPIO8,
    GPIO9,
> where
    I2C: i2c::Instance,
    SPI: spi::Instance,
    UART: uart::Instance,
    PWM_SLICE0: pwm::Slice,
    PWM_SLICE1: pwm::Slice,
    GPIO0: BasePin + i2c::SclPin<I2C>,
    GPIO1: BasePin + i2c::SdaPin<I2C> + ChannelAPin<PWM_SLICE0>,
    GPIO2: BasePin + spi::ClkPin<SPI>,
    GPIO3: BasePin + spi::MisoPin<SPI>,
    GPIO4: BasePin + spi::MosiPin<SPI>,
    GPIO5: BasePin + uart::TxPin<UART>,
    GPIO6: BasePin + uart::RxPin<UART> + ChannelBPin<PWM_SLICE1>,
    GPIO7: BasePin + AdcChannel + AdcPin,
    GPIO8: BasePin + AdcChannel + AdcPin,
    GPIO9: BasePin,
{
    pub gpio0: Peri<'a, GPIO0>,
    pub gpio1: Peri<'a, GPIO1>,
    pub gpio2: Peri<'a, GPIO2>,
    pub gpio3: Peri<'a, GPIO3>,
    pub gpio4: Peri<'a, GPIO4>,
    pub gpio5: Peri<'a, GPIO5>,
    pub gpio6: Peri<'a, GPIO6>,
    pub gpio7: Peri<'a, GPIO7>,
    pub gpio8: Peri<'a, GPIO8>,
    pub gpio9: Peri<'a, GPIO9>,
    pub pwm_slice0: Peri<'a, PWM_SLICE0>,
    pub pwm_slice1: Peri<'a, PWM_SLICE1>,
    _i2c: PhantomData<I2C>,
    _spi: PhantomData<SPI>,
    _uart: PhantomData<UART>,
}

impl<
    'a,
    I2C,
    SPI,
    UART,
    PWM_SLICE0,
    PWM_SLICE1,
    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,
    GPIO5,
    GPIO6,
    GPIO7,
    GPIO8,
    GPIO9,
>
    GpioBank<
        'a,
        I2C,
        SPI,
        UART,
        PWM_SLICE0,
        PWM_SLICE1,
        GPIO0,
        GPIO1,
        GPIO2,
        GPIO3,
        GPIO4,
        GPIO5,
        GPIO6,
        GPIO7,
        GPIO8,
        GPIO9,
    >
where
    I2C: i2c::Instance,
    SPI: spi::Instance,
    UART: uart::Instance,
    PWM_SLICE0: pwm::Slice,
    PWM_SLICE1: pwm::Slice,
    GPIO0: BasePin + i2c::SclPin<I2C>,
    GPIO1: BasePin + i2c::SdaPin<I2C> + ChannelAPin<PWM_SLICE0>,
    GPIO2: BasePin + spi::ClkPin<SPI>,
    GPIO3: BasePin + spi::MisoPin<SPI>,
    GPIO4: BasePin + spi::MosiPin<SPI>,
    GPIO5: BasePin + uart::TxPin<UART>,
    GPIO6: BasePin + uart::RxPin<UART> + ChannelBPin<PWM_SLICE1>,
    GPIO7: BasePin + AdcChannel + AdcPin,
    GPIO8: BasePin + AdcChannel + AdcPin,
    GPIO9: BasePin,
{
    pub fn new(
        gpio0: Peri<'a, GPIO0>,
        gpio1: Peri<'a, GPIO1>,
        gpio2: Peri<'a, GPIO2>,
        gpio3: Peri<'a, GPIO3>,
        gpio4: Peri<'a, GPIO4>,
        gpio5: Peri<'a, GPIO5>,
        gpio6: Peri<'a, GPIO6>,
        gpio7: Peri<'a, GPIO7>,
        gpio8: Peri<'a, GPIO8>,
        gpio9: Peri<'a, GPIO9>,
        pwm_slice0: Peri<'a, PWM_SLICE0>,
        pwm_slice1: Peri<'a, PWM_SLICE1>,
    ) -> Self {
        Self {
            gpio0,
            gpio1,
            gpio2,
            gpio3,
            gpio4,
            gpio5,
            gpio6,
            gpio7,
            gpio8,
            gpio9,
            pwm_slice0,
            pwm_slice1,
            _i2c: PhantomData,
            _spi: PhantomData,
            _uart: PhantomData,
        }
    }
}
