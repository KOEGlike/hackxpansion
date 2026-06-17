use crate::bus::spi::SpiError;
use embassy_rp::gpio::{Input, Level, Output};
use embassy_rp::Peri;

pub struct BitBangSpiBus<'d> {
    clk: Output<'d>,
    mosi: Output<'d>,
    miso: Input<'d>,
    delay_us: u64,
}

impl<'d> BitBangSpiBus<'d> {
    pub fn new(
        clk: Peri<'d, impl embassy_rp::gpio::Pin>,
        mosi: Peri<'d, impl embassy_rp::gpio::Pin>,
        miso: Peri<'d, impl embassy_rp::gpio::Pin>,
        frequency_hz: u32,
    ) -> Self {
        let clk = Output::new(clk, Level::Low);
        let mosi = Output::new(mosi, Level::Low);
        let miso = Input::new(miso, embassy_rp::gpio::Pull::None);

        let delay_us = if frequency_hz > 0 {
            1_000_000 / (2 * frequency_hz as u64)
        } else {
            10
        };

        Self { clk, mosi, miso, delay_us }
    }

    fn transfer_byte_blocking(&mut self, write_byte: u8) -> u8 {
        let mut read_byte = 0u8;
        for i in (0..8).rev() {
            let bit = (write_byte >> i) & 1;
            self.mosi.set_level(Level::from(bit != 0));
            self.clk.set_high();
            embassy_time::block_for(embassy_time::Duration::from_micros(self.delay_us));
            if self.miso.is_high() {
                read_byte |= 1 << i;
            }
            self.clk.set_low();
            embassy_time::block_for(embassy_time::Duration::from_micros(self.delay_us));
        }
        read_byte
    }
}

impl<'d> embedded_hal_1::spi::ErrorType for BitBangSpiBus<'d> {
    type Error = SpiError;
}

impl<'d> embedded_hal_1::spi::SpiBus<u8> for BitBangSpiBus<'d> {
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
        let len = read.len().min(write.len());
        for i in 0..len {
            read[i] = self.transfer_byte_blocking(write[i]);
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
            *byte = self.transfer_byte_blocking(0xFF);
        }
        Ok(())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        for &byte in words {
            self.transfer_byte_blocking(byte);
        }
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        let len = read.len().min(write.len());
        for i in 0..len {
            read[i] = self.transfer_byte_blocking(write[i]);
        }
        Ok(())
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        for v in words.iter_mut() {
            *v = self.transfer_byte_blocking(*v);
        }
        Ok(())
    }
}
