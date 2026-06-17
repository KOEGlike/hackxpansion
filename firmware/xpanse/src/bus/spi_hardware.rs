use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{self, Blocking, Spi};
use embassy_rp::Peri;

use xpanse_driver_api::bus::spi::SpiError;

pub struct HardwareSpiBus<'d> {
    spi: Spi<'d, SPI1, Blocking>,
}

impl<'d> HardwareSpiBus<'d> {
    pub fn new(
        peri: Peri<'d, SPI1>,
        clk: Peri<'d, impl embassy_rp::spi::ClkPin<SPI1> + 'd>,
        mosi: Peri<'d, impl embassy_rp::spi::MosiPin<SPI1> + 'd>,
        miso: Peri<'d, impl embassy_rp::spi::MisoPin<SPI1> + 'd>,
        config: spi::Config,
    ) -> Self {
        let spi = Spi::new_blocking(peri, clk, mosi, miso, config);
        Self { spi }
    }
}

// Delegate embedded-hal ErrorType to inner Spi
impl<'d> embedded_hal_1::spi::ErrorType for HardwareSpiBus<'d> {
    type Error = SpiError;
}

impl<'d> embedded_hal_1::spi::SpiBus<u8> for HardwareSpiBus<'d> {
    fn flush(&mut self) -> Result<(), SpiError> {
        self.spi.flush().map_err(|e| e.into())
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi.blocking_read(words).map_err(|e| e.into())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        self.spi.blocking_write(words).map_err(|e| e.into())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.spi.blocking_transfer(read, write).map_err(|e| e.into())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi.blocking_transfer_in_place(words).map_err(|e| e.into())
    }
}

impl<'d> embedded_hal_async::spi::SpiBus<u8> for HardwareSpiBus<'d> {
    async fn flush(&mut self) -> Result<(), SpiError> {
        self.spi.flush().map_err(|e| e.into())
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi.blocking_read(words).map_err(|e| e.into())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        self.spi.blocking_write(words).map_err(|e| e.into())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.spi.blocking_transfer(read, write).map_err(|e| e.into())
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi.blocking_transfer_in_place(words).map_err(|e| e.into())
    }
}
