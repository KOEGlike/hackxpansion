use crate::bus::uart::UartError;
use embassy_rp::Peri;
use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::uart::{Async, Config, Instance, InterruptHandler, Uart};

pub struct HardwareUartBus<'d> {
    uart: Uart<'d, Async>,
}

impl<'d> HardwareUartBus<'d> {
    pub fn new<I, TxDma, RxDma>(
        peri: Peri<'d, I>,
        tx: Peri<'d, impl embassy_rp::uart::TxPin<I> + 'd>,
        rx: Peri<'d, impl embassy_rp::uart::RxPin<I> + 'd>,
        irq: impl Binding<I::Interrupt, InterruptHandler<I>>
        + Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
        + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
        + 'd,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        config: Config,
    ) -> Self
    where
        I: Instance,
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
    {
        let uart = Uart::new(peri, tx, rx, irq, tx_dma, rx_dma, config);
        Self { uart }
    }
}

impl<'d> embedded_io_async::ErrorType for HardwareUartBus<'d> {
    type Error = UartError;
}

impl<'d> embedded_io_async::Read for HardwareUartBus<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, UartError> {
        self.uart.read(buf).await.map_err(UartError::from)?;
        Ok(buf.len())
    }
}

impl<'d> embedded_io_async::Write for HardwareUartBus<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, UartError> {
        self.uart.write(buf).await.map_err(UartError::from)?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), UartError> {
        Ok(())
    }
}
