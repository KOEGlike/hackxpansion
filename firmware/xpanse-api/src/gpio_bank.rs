//! Strongly typed GPIO banks assigned to expansion modules.
//!
//! Each bank contains ten GPIOs and three PWM slices. [`BankPins`] records the
//! fixed hardware roles available on those pins, allowing driver code to remain
//! generic over the physical module slot while preserving compile-time pin
//! checks.

#![allow(non_camel_case_types)]

use core::{
    marker::PhantomData,
    panic::{RefUnwindSafe, UnwindSafe},
};

use embassy_rp::{
    Peri, PeripheralType,
    adc::{AdcChannel, AdcPin},
    gpio::{AnyPin, Pin},
    i2c,
    pio::PioPin,
    pwm::{self, ChannelAPin, ChannelBPin},
    spi, uart,
};

use crate::bus::allocator::{I2cHw, SpiHw, UartHw};

/// Common bounds required of every GPIO in an expansion bank.
///
/// This trait is implemented automatically for compatible RP235x pin types and
/// is not intended to be implemented by drivers.
pub trait BasePin:
    RefUnwindSafe
    + Send
    + Sync
    + Unpin
    + UnwindSafe
    + Copy
    + Pin
    + PioPin
    + PeripheralType
    + Into<AnyPin>
{
}

impl<T> BasePin for T where
    T: RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + Copy
        + Pin
        + PioPin
        + PeripheralType
        + Into<AnyPin>
{
}

/// Type-level description of one expansion GPIO bank.
///
/// The firmware normally uses the tuple implementation supplied by this crate;
/// drivers only need a generic `G: BankPins` bound.
pub trait BankPins {
    /// Hardware I2C instance connected to GPIO 0 and GPIO 1.
    type I2C: i2c::Instance + 'static + I2cHw;
    /// Hardware SPI instance connected to GPIO 2 through GPIO 4.
    type SPI: spi::Instance + 'static + SpiHw;
    /// Hardware UART instance connected to GPIO 5 and GPIO 6.
    type UART: uart::Instance + 'static + UartHw;
    /// PWM slice connected to GPIO 7 and GPIO 8.
    type PWM_SLICE0: pwm::Slice + 'static;
    /// PWM slice connected to GPIO 5 and GPIO 6.
    type PWM_SLICE1: pwm::Slice + 'static;
    /// PWM slice connected to GPIO 2.
    type PWM_SLICE2: pwm::Slice + 'static;
    /// GPIO 0, usable as I2C SCL.
    type GPIO0: BasePin + i2c::SclPin<Self::I2C>;
    /// GPIO 1, usable as I2C SDA.
    type GPIO1: BasePin + i2c::SdaPin<Self::I2C>;
    /// GPIO 2, usable as SPI clock or PWM channel A.
    type GPIO2: BasePin + spi::ClkPin<Self::SPI> + ChannelAPin<Self::PWM_SLICE2>;
    /// GPIO 3, usable as SPI MISO.
    type GPIO3: BasePin + spi::MisoPin<Self::SPI>;
    /// GPIO 4, usable as SPI MOSI.
    type GPIO4: BasePin + spi::MosiPin<Self::SPI>;
    /// GPIO 5, usable as UART TX or PWM channel A.
    type GPIO5: BasePin + uart::TxPin<Self::UART> + ChannelAPin<Self::PWM_SLICE1>;
    /// GPIO 6, usable as UART RX or PWM channel B.
    type GPIO6: BasePin + uart::RxPin<Self::UART> + ChannelBPin<Self::PWM_SLICE1>;
    /// GPIO 7, usable as ADC input or PWM channel A.
    type GPIO7: BasePin + AdcChannel + AdcPin + ChannelAPin<Self::PWM_SLICE0>;
    /// GPIO 8, usable as ADC input or PWM channel B.
    type GPIO8: BasePin + AdcChannel + AdcPin + ChannelBPin<Self::PWM_SLICE0>;
    /// GPIO 9, available as general-purpose GPIO.
    type GPIO9: BasePin;
}

/// Owned peripheral tokens for one expansion module slot.
///
/// A [`Driver`](crate::driver::Driver) consumes this value during module
/// initialization. Individual fields can then be moved into GPIO interfaces or
/// bus constructors.
pub struct GpioBank<G: BankPins> {
    /// GPIO 0, with the bank's I2C SCL role.
    pub gpio0: Peri<'static, G::GPIO0>,
    /// GPIO 1, with the bank's I2C SDA role.
    pub gpio1: Peri<'static, G::GPIO1>,
    /// GPIO 2, with SPI clock and PWM roles.
    pub gpio2: Peri<'static, G::GPIO2>,
    /// GPIO 3, with the SPI MISO role.
    pub gpio3: Peri<'static, G::GPIO3>,
    /// GPIO 4, with the SPI MOSI role.
    pub gpio4: Peri<'static, G::GPIO4>,
    /// GPIO 5, with UART TX and PWM roles.
    pub gpio5: Peri<'static, G::GPIO5>,
    /// GPIO 6, with UART RX and PWM roles.
    pub gpio6: Peri<'static, G::GPIO6>,
    /// GPIO 7, with ADC and PWM roles.
    pub gpio7: Peri<'static, G::GPIO7>,
    /// GPIO 8, with ADC and PWM roles.
    pub gpio8: Peri<'static, G::GPIO8>,
    /// GPIO 9, available as general-purpose GPIO.
    pub gpio9: Peri<'static, G::GPIO9>,
    /// PWM slice shared by GPIO 7 and GPIO 8.
    pub pwm_slice0: Peri<'static, G::PWM_SLICE0>,
    /// PWM slice shared by GPIO 5 and GPIO 6.
    pub pwm_slice1: Peri<'static, G::PWM_SLICE1>,
    /// PWM slice connected to GPIO 2.
    pub pwm_slice2: Peri<'static, G::PWM_SLICE2>,
    _phantom: PhantomData<G>,
}

impl<
    I2C: i2c::Instance + 'static + I2cHw,
    SPI: spi::Instance + 'static + SpiHw,
    UART: uart::Instance + 'static + UartHw,
    PWM_SLICE0: pwm::Slice + 'static,
    PWM_SLICE1: pwm::Slice + 'static,
    PWM_SLICE2: pwm::Slice + 'static,
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
    I2C: i2c::Instance + 'static + I2cHw,
    SPI: spi::Instance + 'static + SpiHw,
    UART: uart::Instance + 'static + UartHw,
    PWM_SLICE0: pwm::Slice + 'static,
    PWM_SLICE1: pwm::Slice + 'static,
    PWM_SLICE2: pwm::Slice + 'static,
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
    GpioBank<(
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
    )>
{
    /// Builds a bank from the peripheral tokens assigned to one module slot.
    ///
    /// This is normally called by platform firmware after splitting the board's
    /// peripherals. Module drivers receive the completed bank through
    /// [`Driver::create`](crate::driver::Driver::create).
    pub fn new(
        gpio0: Peri<'static, GPIO0>,
        gpio1: Peri<'static, GPIO1>,
        gpio2: Peri<'static, GPIO2>,
        gpio3: Peri<'static, GPIO3>,
        gpio4: Peri<'static, GPIO4>,
        gpio5: Peri<'static, GPIO5>,
        gpio6: Peri<'static, GPIO6>,
        gpio7: Peri<'static, GPIO7>,
        gpio8: Peri<'static, GPIO8>,
        gpio9: Peri<'static, GPIO9>,
        pwm_slice0: Peri<'static, PWM_SLICE0>,
        pwm_slice1: Peri<'static, PWM_SLICE1>,
        pwm_slice2: Peri<'static, PWM_SLICE2>,
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
