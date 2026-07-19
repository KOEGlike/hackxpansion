use crate::bus::uart::UartError;
use embassy_rp::Peri;
use embassy_rp::pio::{Common, PioPin, StateMachine};
use embassy_rp::pio_programs::uart::{PioUartRx, PioUartRxProgram, PioUartTx, PioUartTxProgram};
use embassy_time::{Duration, Instant, TICK_HZ, Timer};

pub struct PioUartBus<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize> {
    tx: PioUartTx<'d, PIO, SM_TX>,
    rx: PioUartRx<'d, PIO, SM_RX>,
    // Keep the loaded programs alive for the bus's lifetime so the PIO
    // instruction memory stays reserved and is never double-allocated.
    _tx_program: PioUartTxProgram<'d, PIO>,
    _rx_program: PioUartRxProgram<'d, PIO>,
    tx_idle_at: Instant,
    frame_duration: Duration,
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize>
    PioUartBus<'d, PIO, SM_TX, SM_RX>
{
    pub fn validate_baud(baud_rate: u32) -> Result<(), UartError> {
        baud_parameters(baud_rate).map(|_| ())
    }

    pub fn new(
        common: &mut Common<'d, PIO>,
        sm_tx: StateMachine<'d, PIO, SM_TX>,
        sm_rx: StateMachine<'d, PIO, SM_RX>,
        tx_pin: Peri<'d, impl PioPin>,
        rx_pin: Peri<'d, impl PioPin>,
        baud_rate: u32,
    ) -> Result<Self, UartError> {
        let (programmed_baud, actual_baud) = baud_parameters(baud_rate)?;

        let tx_program = PioUartTxProgram::new(common);
        let rx_program = PioUartRxProgram::new(common);
        let tx = PioUartTx::new(programmed_baud, common, sm_tx, tx_pin, &tx_program);
        let rx = PioUartRx::new(programmed_baud, common, sm_rx, rx_pin, &rx_program);
        let frame_ticks = TICK_HZ.saturating_mul(10).saturating_add(actual_baud - 1) / actual_baud;
        Ok(Self {
            tx,
            rx,
            _tx_program: tx_program,
            _rx_program: rx_program,
            tx_idle_at: Instant::MIN,
            frame_duration: Duration::from_ticks(frame_ticks.max(1)),
        })
    }
}

fn baud_parameters(baud_rate: u32) -> Result<(u32, u64), UartError> {
    let clock = embassy_rp::clocks::clk_sys_freq() as u64;
    let denominator = 8u64.saturating_mul(baud_rate as u64);
    if denominator == 0 || denominator > clock || clock > denominator.saturating_mul(65_536) {
        return Err(UartError::InvalidBaudRate);
    }

    let divider = (clock + denominator / 2) / denominator;
    if !(1..=65_536).contains(&divider) {
        return Err(UartError::InvalidBaudRate);
    }

    let programmed_baud = (clock / (8 * divider)) as u32;
    if programmed_baud == 0 {
        return Err(UartError::InvalidBaudRate);
    }
    let locked_divider = clock / (8 * programmed_baud as u64);
    if locked_divider != divider || !(1..=65_536).contains(&locked_divider) {
        return Err(UartError::InvalidBaudRate);
    }

    let actual_baud = clock / (8 * locked_divider);
    let difference = actual_baud.abs_diff(baud_rate as u64);
    if difference.saturating_mul(1_000) > (baud_rate as u64).saturating_mul(25) {
        return Err(UartError::InvalidBaudRate);
    }

    Ok((programmed_baud, actual_baud))
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
        if buf.is_empty() {
            return Ok(0);
        }
        self.rx
            .read(&mut buf[..1])
            .await
            .map_err(|_| UartError::Other)
    }
}

impl<'d, PIO: embassy_rp::pio::Instance, const SM_TX: usize, const SM_RX: usize>
    embedded_io_async::Write for PioUartBus<'d, PIO, SM_TX, SM_RX>
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, UartError> {
        if buf.is_empty() {
            return Ok(0);
        }

        let transmission_start = self.tx_idle_at.max(Instant::now());
        let frames = (buf.len() as u64).saturating_add(1);
        let transmission_time =
            Duration::from_ticks(self.frame_duration.as_ticks().saturating_mul(frames));
        self.tx_idle_at = transmission_start.saturating_add(transmission_time);
        let written = self.tx.write(buf).await.map_err(|_| UartError::Other)?;
        Ok(written)
    }

    async fn flush(&mut self) -> Result<(), UartError> {
        self.tx.flush().await.map_err(|_| UartError::Other)?;
        Timer::at(self.tx_idle_at).await;
        Ok(())
    }
}
