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

pub trait BankPins {
    type I2C: i2c::Instance;
    type SPI: spi::Instance;
    type UART: uart::Instance;
    type PWM_SLICE0: pwm::Slice;
    type PWM_SLICE1: pwm::Slice;
    type PWM_SLICE2: pwm::Slice;
    type GPIO0: BasePin + i2c::SclPin<Self::I2C>;
    type GPIO1: BasePin + i2c::SdaPin<Self::I2C>;
    type GPIO2: BasePin + spi::ClkPin<Self::SPI> + ChannelAPin<Self::PWM_SLICE2>;
    type GPIO3: BasePin + spi::MisoPin<Self::SPI>;
    type GPIO4: BasePin + spi::MosiPin<Self::SPI>;
    type GPIO5: BasePin + uart::TxPin<Self::UART> + ChannelAPin<Self::PWM_SLICE1>;
    type GPIO6: BasePin + uart::RxPin<Self::UART> + ChannelBPin<Self::PWM_SLICE1>;
    type GPIO7: BasePin + AdcChannel + AdcPin + ChannelAPin<Self::PWM_SLICE0>;
    type GPIO8: BasePin + AdcChannel + AdcPin + ChannelBPin<Self::PWM_SLICE0>;
    type GPIO9: BasePin;
}

pub struct GpioBank<'a, G: BankPins> {
    pub gpio0: Peri<'a, G::GPIO0>,
    pub gpio1: Peri<'a, G::GPIO1>,
    pub gpio2: Peri<'a, G::GPIO2>,
    pub gpio3: Peri<'a, G::GPIO3>,
    pub gpio4: Peri<'a, G::GPIO4>,
    pub gpio5: Peri<'a, G::GPIO5>,
    pub gpio6: Peri<'a, G::GPIO6>,
    pub gpio7: Peri<'a, G::GPIO7>,
    pub gpio8: Peri<'a, G::GPIO8>,
    pub gpio9: Peri<'a, G::GPIO9>,
    pub pwm_slice0: Peri<'a, G::PWM_SLICE0>,
    pub pwm_slice1: Peri<'a, G::PWM_SLICE1>,
    pub pwm_slice2: Peri<'a, G::PWM_SLICE2>,
    _phantom: PhantomData<G>,
}

impl<
    I2C: i2c::Instance,
    SPI: spi::Instance,
    UART: uart::Instance,
    PWM_SLICE0: pwm::Slice,
    PWM_SLICE1: pwm::Slice,
    PWM_SLICE2: pwm::Slice,
    GPIO0: BasePin + i2c::SclPin<I2C>,
    GPIO1: BasePin + i2c::SdaPin<I2C>,
    GPIO2: BasePin + spi::ClkPin<SPI> + ChannelAPin<PWM_SLICE2>,
    GPIO3: BasePin + spi::MisoPin<SPI>,
    GPIO4: BasePin + spi::MosiPin<SPI>,
    GPIO5: BasePin + uart::TxPin<UART> + ChannelAPin<PWM_SLICE1>,
    GPIO6: BasePin + uart::RxPin<UART> + ChannelBPin<PWM_SLICE1>,
    GPIO7: BasePin + AdcChannel + AdcPin + ChannelAPin<PWM_SLICE0>,
    GPIO8: BasePin + AdcChannel + AdcPin + ChannelBPin<PWM_SLICE0>,
    GPIO9: BasePin,
> BankPins
    for (
        I2C, SPI, UART, PWM_SLICE0, PWM_SLICE1, PWM_SLICE2, GPIO0, GPIO1, GPIO2, GPIO3, GPIO4,
        GPIO5, GPIO6, GPIO7, GPIO8, GPIO9,
    )
{
    type I2C = I2C;
    type SPI = SPI;
    type UART = UART;
    type PWM_SLICE0 = PWM_SLICE0;
    type PWM_SLICE1 = PWM_SLICE1;
    type PWM_SLICE2 = PWM_SLICE2;
    type GPIO0 = GPIO0;
    type GPIO1 = GPIO1;
    type GPIO2 = GPIO2;
    type GPIO3 = GPIO3;
    type GPIO4 = GPIO4;
    type GPIO5 = GPIO5;
    type GPIO6 = GPIO6;
    type GPIO7 = GPIO7;
    type GPIO8 = GPIO8;
    type GPIO9 = GPIO9;
}

impl<
    'a,
    I2C: i2c::Instance,
    SPI: spi::Instance,
    UART: uart::Instance,
    PWM_SLICE0: pwm::Slice,
    PWM_SLICE1: pwm::Slice,
    PWM_SLICE2: pwm::Slice,
    GPIO0: BasePin + i2c::SclPin<I2C>,
    GPIO1: BasePin + i2c::SdaPin<I2C>,
    GPIO2: BasePin + spi::ClkPin<SPI> + ChannelAPin<PWM_SLICE2>,
    GPIO3: BasePin + spi::MisoPin<SPI>,
    GPIO4: BasePin + spi::MosiPin<SPI>,
    GPIO5: BasePin + uart::TxPin<UART> + ChannelAPin<PWM_SLICE1>,
    GPIO6: BasePin + uart::RxPin<UART> + ChannelBPin<PWM_SLICE1>,
    GPIO7: BasePin + AdcChannel + AdcPin + ChannelAPin<PWM_SLICE0>,
    GPIO8: BasePin + AdcChannel + AdcPin + ChannelBPin<PWM_SLICE0>,
    GPIO9: BasePin,
>
    GpioBank<
        'a,
        (
            I2C,
            SPI,
            UART,
            PWM_SLICE0,
            PWM_SLICE1,
            PWM_SLICE2,
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
        ),
    >
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
        pwm_slice2: Peri<'a, PWM_SLICE2>,
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
            pwm_slice2,
            _phantom: PhantomData,
        }
    }
}
