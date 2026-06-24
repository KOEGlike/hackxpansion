use crate::bus::uart::UartError;
use embassy_rp::pio::{Common, PioPin, StateMachine};
use embassy_rp::pio_programs::uart::{PioUartRx, PioUartRxProgram, PioUartTx, PioUartTxProgram};
use embassy_rp::Peri;

pub struct PioUartBus<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize> {
    tx: PioUartTx<'d, PIO, SM_TX>,
    rx: PioUartRx<'d, PIO, SM_RX>,
    // Keep the loaded programs alive for the bus's lifetime so the PIO
    // instruction memory stays reserved and is never double-allocated.
    _tx_program: PioUartTxProgram<'d, PIO>,
    _rx_program: PioUartRxProgram<'d, PIO>,
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize>
    PioUartBus<'d, PIO, SM_TX, SM_RX>
{
    pub fn new(
        common: &mut Common<'d, PIO>,
        sm_tx: StateMachine<'d, PIO, SM_TX>,
        sm_rx: StateMachine<'d, PIO, SM_RX>,
        tx_pin: Peri<'d, impl PioPin>,
        rx_pin: Peri<'d, impl PioPin>,
        baud_rate: u32,
    ) -> Self {
        let tx_program = PioUartTxProgram::new(common);
        let rx_program = PioUartRxProgram::new(common);
        let tx = PioUartTx::new(baud_rate, common, sm_tx, tx_pin, &tx_program);
        let rx = PioUartRx::new(baud_rate, common, sm_rx, rx_pin, &rx_program);
        Self {
            tx,
            rx,
            _tx_program: tx_program,
            _rx_program: rx_program,
        }
    }
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize>
    embedded_io_async::ErrorType for PioUartBus<'d, PIO, SM_TX, SM_RX>
{
    type Error = UartError;
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize>
    embedded_io_async::Read for PioUartBus<'d, PIO, SM_TX, SM_RX>
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, UartError> {
        self.rx.read(buf).await.map_err(|_| UartError::Other)
    }
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize>
    embedded_io_async::Write for PioUartBus<'d, PIO, SM_TX, SM_RX>
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, UartError> {
        self.tx.write(buf).await.map_err(|_| UartError::Other)
    }

    async fn flush(&mut self) -> Result<(), UartError> {
        self.tx.flush().await.map_err(|_| UartError::Other)
    }
}
