//! RP235x hardware UART backend using DMA and interrupts.
//!
//! The platform normally constructs these buses through
//! [`BusAllocator::create_uart_hardware`](crate::bus::allocator::BusAllocator::create_uart_hardware).
//! Reads return at most one byte per call, matching the streaming contract of
//! all [`UartBusHandle`](crate::bus::uart::UartBusHandle) backends.
use crate::bus::uart::UartError;
use embassy_rp::Peri;
use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::uart::{Async, Config, Instance, InterruptHandler, Uart};
use embassy_time::{Duration, Timer};

/// DMA-backed RP235x UART bus implementing the async [`Read`] and [`Write`]
/// traits.
///
/// [`Read`]: crate::reexports::embedded_io_async::Read
/// [`Write`]: crate::reexports::embedded_io_async::Write
pub struct HardwareUartBus<'d> {
    uart: Uart<'d, Async>,
}

impl<'d> HardwareUartBus<'d> {
    /// Validates a UART configuration against the running peripheral clock.
    ///
    /// Returns [`UartError::InvalidBaudRate`] when the requested rate falls
    /// outside the range the RP235x divisor can represent.
    pub fn validate_config(config: &Config) -> Result<(), UartError> {
        let clock = embassy_rp::clocks::clk_peri_freq() as u64;
        let baudrate = config.baudrate as u64;
        let minimum_baudrate = clock.div_ceil(16 * 65_535);
        if baudrate < minimum_baudrate || baudrate > clock / 16 {
            return Err(UartError::InvalidBaudRate);
        }
        Ok(())
    }

    /// Builds a UART bus from a previously validated configuration.
    ///
    /// `tx` and `rx` are role-checked against `I` at compile time, and `irq`
    /// must bind both the UART interrupt and the TX/RX DMA interrupts.
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
    ) -> Result<Self, UartError>
    where
        I: Instance,
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
    {
        Self::validate_config(&config)?;
        let uart = Uart::new(peri, tx, rx, irq, tx_dma, rx_dma, config);
        Ok(Self { uart })
    }
}

impl<'d> embedded_io_async::ErrorType for HardwareUartBus<'d> {
    type Error = UartError;
}

impl<'d> embedded_io_async::Read for HardwareUartBus<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, UartError> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.uart
            .read(&mut buf[..1])
            .await
            .map_err(UartError::from)?;
        Ok(1)
    }
}

impl<'d> embedded_io_async::Write for HardwareUartBus<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, UartError> {
        self.uart.write(buf).await.map_err(UartError::from)?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), UartError> {
        while self.uart.busy() {
            Timer::after(Duration::from_micros(1)).await;
        }
        Ok(())
    }
}
