//! RP235x hardware SPI backend driven by DMA.
//!
//! The platform normally constructs these buses through
//! [`BusAllocator::create_spi_hardware`](crate::bus::allocator::BusAllocator::create_spi_hardware).
//!
use crate::bus::spi::SpiError;
use embassy_rp::Peri;
use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::spi::{self, Async, Instance, Spi};

/// DMA-backed RP235x SPI bus implementing both
/// `embedded_hal::spi::SpiBus` and `embedded_hal_async::spi::SpiBus`
/// for `SpiBusHandle`.
pub struct HardwareSpiBus<'d, I: Instance> {
    spi: Spi<'d, I, Async>,
}

impl<'d, I: Instance> HardwareSpiBus<'d, I> {
    /// Validates a configuration against the running peripheral clock and DMA
    /// constraints.
    ///
    /// Returns [`SpiError::InvalidFrequency`] when the divider ratio falls
    /// outside the range the RP235x hardware can represent.
    pub fn validate_config(config: &spi::Config) -> Result<(), SpiError> {
        let clock = embassy_rp::clocks::clk_peri_freq() as u64;
        let frequency = config.frequency as u64;
        if frequency == 0 || frequency > clock / 2 {
            return Err(SpiError::InvalidFrequency);
        }
        let ratio = clock.div_ceil(2 * frequency);
        if ratio > 127 * 256 {
            return Err(SpiError::InvalidFrequency);
        }
        Ok(())
    }

    pub fn new<TxDma, RxDma, Irq>(
        peri: Peri<'d, I>,
        clk: Peri<'d, impl embassy_rp::spi::ClkPin<I> + 'd>,
        mosi: Peri<'d, impl embassy_rp::spi::MosiPin<I> + 'd>,
        miso: Peri<'d, impl embassy_rp::spi::MisoPin<I> + 'd>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        irq: Irq,
        config: spi::Config,
    ) -> Result<Self, SpiError>
    where
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'd,
    {
        Self::validate_config(&config)?;
        let spi = Spi::new(peri, clk, mosi, miso, tx_dma, rx_dma, irq, config);
        Ok(Self { spi })
    }
}

impl<'d, I: Instance> embedded_hal::spi::ErrorType for HardwareSpiBus<'d, I> {
    type Error = SpiError;
}

impl<'d, I: Instance> embedded_hal::spi::SpiBus<u8> for HardwareSpiBus<'d, I> {
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
        self.spi
            .blocking_transfer(read, write)
            .map_err(|e| e.into())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi
            .blocking_transfer_in_place(words)
            .map_err(|e| e.into())
    }
}

impl<'d, I: Instance> embedded_hal_async::spi::SpiBus<u8> for HardwareSpiBus<'d, I> {
    async fn flush(&mut self) -> Result<(), SpiError> {
        self.spi.flush().map_err(|e| e.into())
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi.read(words).await.map_err(|e| e.into())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        self.spi.write(words).await.map_err(|e| e.into())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.spi.transfer(read, write).await.map_err(|e| e.into())
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.spi
            .transfer_in_place(words)
            .await
            .map_err(|e| e.into())
    }
}
