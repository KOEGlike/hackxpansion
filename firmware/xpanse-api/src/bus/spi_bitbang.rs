//! GPIO bit-banged SPI master backend.
//!
//! The platform normally constructs these buses through
//! [`BusAllocator::create_spi_bitbang`](crate::bus::allocator::BusAllocator::create_spi_bitbang).
//! No chip-select line is managed here; use `embedded-hal-bus` or similar to
//! share the bus with a device driver.
use crate::bus::spi::SpiError;
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::spi::{Config, Phase, Polarity};
use embassy_time::{Duration, TICK_HZ, Timer};

/// GPIO bit-banged SPI bus using timer-driven delays.
///
/// Implements both blocking and asynchronous SPI traits. Supported polarities
/// and phases are determined by the RP235x GPIO pad capabilities.
pub struct BitBangSpiBus<'d> {
    clk: Output<'d>,
    mosi: Output<'d>,
    miso: Input<'d>,
    half_bit: Duration,
    idle_level: Level,
    active_level: Level,
    capture_on_first_transition: bool,
}

struct ClockIdleGuard<'a, 'd> {
    clock: &'a mut Output<'d>,
    idle_level: Level,
}

impl Drop for ClockIdleGuard<'_, '_> {
    fn drop(&mut self) {
        self.clock.set_level(self.idle_level);
    }
}

impl<'d> BitBangSpiBus<'d> {
    /// Validates an SPI configuration against timer constraints.
    ///
    /// Returns [`SpiError::InvalidFrequency`] for a zero clock or a clock above
    /// half the timer tick rate.
    pub fn validate_config(config: &Config) -> Result<(), SpiError> {
        if config.frequency == 0 || config.frequency as u64 > TICK_HZ / 2 {
            return Err(SpiError::InvalidFrequency);
        }
        Ok(())
    }

    /// Creates a bit-banged SPI master from three GPIO pins and a configuration.
    ///
    /// # Errors
    ///
    /// Returns [`SpiError::InvalidFrequency`] for an unsupported clock speed.
    pub fn new(
        clk: Peri<'d, impl embassy_rp::gpio::Pin>,
        mosi: Peri<'d, impl embassy_rp::gpio::Pin>,
        miso: Peri<'d, impl embassy_rp::gpio::Pin>,
        config: Config,
    ) -> Result<Self, SpiError> {
        Self::validate_config(&config)?;

        let idle_level = match config.polarity {
            Polarity::IdleLow => Level::Low,
            Polarity::IdleHigh => Level::High,
        };
        let active_level = match idle_level {
            Level::Low => Level::High,
            Level::High => Level::Low,
        };
        let clk = Output::new(clk, idle_level);
        let mosi = Output::new(mosi, Level::Low);
        let miso = Input::new(miso, embassy_rp::gpio::Pull::None);
        let divisor = 2 * config.frequency as u64;
        let half_bit_ns = 1_000_000_000u64.div_ceil(divisor);

        Ok(Self {
            clk,
            mosi,
            miso,
            half_bit: Duration::from_nanos(half_bit_ns),
            idle_level,
            active_level,
            capture_on_first_transition: matches!(config.phase, Phase::CaptureOnFirstTransition),
        })
    }

    fn transfer_byte_blocking(&mut self, write_byte: u8) -> u8 {
        let mut read_byte = 0u8;
        for i in (0..8).rev() {
            let bit = (write_byte >> i) & 1;
            if self.capture_on_first_transition {
                self.mosi.set_level(Level::from(bit != 0));
                embassy_time::block_for(self.half_bit);
                self.clk.set_level(self.active_level);
                if self.miso.is_high() {
                    read_byte |= 1 << i;
                }
                embassy_time::block_for(self.half_bit);
                self.clk.set_level(self.idle_level);
            } else {
                self.clk.set_level(self.active_level);
                self.mosi.set_level(Level::from(bit != 0));
                embassy_time::block_for(self.half_bit);
                self.clk.set_level(self.idle_level);
                if self.miso.is_high() {
                    read_byte |= 1 << i;
                }
                embassy_time::block_for(self.half_bit);
            }
        }
        read_byte
    }

    async fn transfer_byte(&mut self, write_byte: u8) -> u8 {
        let clock = &mut self.clk;
        let mosi = &mut self.mosi;
        let miso = &self.miso;
        let clock = ClockIdleGuard {
            clock,
            idle_level: self.idle_level,
        };
        let mut read_byte = 0u8;
        for i in (0..8).rev() {
            let bit = (write_byte >> i) & 1;
            if self.capture_on_first_transition {
                mosi.set_level(Level::from(bit != 0));
                Timer::after(self.half_bit).await;
                clock.clock.set_level(self.active_level);
                if miso.is_high() {
                    read_byte |= 1 << i;
                }
                Timer::after(self.half_bit).await;
                clock.clock.set_level(self.idle_level);
            } else {
                clock.clock.set_level(self.active_level);
                mosi.set_level(Level::from(bit != 0));
                Timer::after(self.half_bit).await;
                clock.clock.set_level(self.idle_level);
                if miso.is_high() {
                    read_byte |= 1 << i;
                }
                Timer::after(self.half_bit).await;
            }
        }
        read_byte
    }
}

impl<'d> embedded_hal::spi::ErrorType for BitBangSpiBus<'d> {
    type Error = SpiError;
}

impl<'d> embedded_hal::spi::SpiBus<u8> for BitBangSpiBus<'d> {
    fn flush(&mut self) -> Result<(), SpiError> {
        Ok(())
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        for byte in words.iter_mut() {
            *byte = self.transfer_byte_blocking(0xFF);
        }
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        for &byte in words {
            self.transfer_byte_blocking(byte);
        }
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        let len = read.len().max(write.len());
        for i in 0..len {
            let read_byte = self.transfer_byte_blocking(write.get(i).copied().unwrap_or(0xFF));
            if let Some(byte) = read.get_mut(i) {
                *byte = read_byte;
            }
        }
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        for v in words.iter_mut() {
            *v = self.transfer_byte_blocking(*v);
        }
        Ok(())
    }
}

impl<'d> embedded_hal_async::spi::SpiBus<u8> for BitBangSpiBus<'d> {
    async fn flush(&mut self) -> Result<(), SpiError> {
        Ok(())
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        for byte in words.iter_mut() {
            *byte = self.transfer_byte(0xFF).await;
        }
        Ok(())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        for &byte in words {
            self.transfer_byte(byte).await;
        }
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        let len = read.len().max(write.len());
        for i in 0..len {
            let read_byte = self
                .transfer_byte(write.get(i).copied().unwrap_or(0xFF))
                .await;
            if let Some(byte) = read.get_mut(i) {
                *byte = read_byte;
            }
        }
        Ok(())
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        for v in words.iter_mut() {
            *v = self.transfer_byte(*v).await;
        }
        Ok(())
    }
}
