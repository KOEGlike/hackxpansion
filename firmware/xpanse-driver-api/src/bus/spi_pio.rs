use crate::bus::spi::SpiError;
use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::pio::{Common, PioPin, StateMachine};
use embassy_rp::pio_programs::spi as pio_spi;
use embassy_rp::spi::{self, Async};
use embassy_rp::Peri;

pub struct PioSpiBus<'d, PIO: embassy_rp::pio::Instance, const SM: usize> {
    spi: pio_spi::Spi<'d, PIO, SM, Async>,
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM: usize> PioSpiBus<'d, PIO, SM> {
    pub fn new<TxDma, RxDma, Irq>(
        common: &mut Common<'d, PIO>,
        sm: StateMachine<'d, PIO, SM>,
        clk: Peri<'d, impl PioPin>,
        mosi: Peri<'d, impl PioPin>,
        miso: Peri<'d, impl PioPin>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        irq: Irq,
        config: spi::Config,
    ) -> Self
    where
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'd,
    {
        let spi = pio_spi::Spi::new(common, sm, clk, mosi, miso, tx_dma, rx_dma, irq, config);
        Self { spi }
    }
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM: usize> embedded_hal_1::spi::ErrorType
    for PioSpiBus<'d, PIO, SM>
{
    type Error = SpiError;
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM: usize> embedded_hal_1::spi::SpiBus<u8>
    for PioSpiBus<'d, PIO, SM>
{
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

impl<'d, PIO: embassy_rp::pio::Instance, const SM: usize> embedded_hal_async::spi::SpiBus<u8>
    for PioSpiBus<'d, PIO, SM>
{
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
        self.spi.transfer_in_place(words).await.map_err(|e| e.into())
    }
}
