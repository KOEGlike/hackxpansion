use crate::bus::spi::SpiError;
use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::spi::{self, Async, Instance, Spi};
use embassy_rp::Peri;

pub struct HardwareSpiBus<'d, I: Instance> {
    spi: Spi<'d, I, Async>,
}

impl<'d, I: Instance> HardwareSpiBus<'d, I> {
    pub fn new<TxDma, RxDma, Irq>(
        peri: Peri<'d, I>,
        clk: Peri<'d, impl embassy_rp::spi::ClkPin<I> + 'd>,
        mosi: Peri<'d, impl embassy_rp::spi::MosiPin<I> + 'd>,
        miso: Peri<'d, impl embassy_rp::spi::MisoPin<I> + 'd>,
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
        let spi = Spi::new(peri, clk, mosi, miso, tx_dma, rx_dma, irq, config);
        Self { spi }
    }
}

impl<'d, I: Instance> embedded_hal_1::spi::ErrorType for HardwareSpiBus<'d, I> {
    type Error = SpiError;
}

impl<'d, I: Instance> embedded_hal_1::spi::SpiBus<u8> for HardwareSpiBus<'d, I> {
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
        self.spi.transfer_in_place(words).await.map_err(|e| e.into())
    }
}
