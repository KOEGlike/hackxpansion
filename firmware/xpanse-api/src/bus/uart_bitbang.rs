use crate::bus::uart::UartError;
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::{Duration, TICK_HZ, Timer};

pub struct BitBangUartBus<'d> {
    tx: Output<'d>,
    rx: Input<'d>,
    bit_duration: Duration,
}

struct TxIdleGuard<'a, 'd>(&'a mut Output<'d>);

impl Drop for TxIdleGuard<'_, '_> {
    fn drop(&mut self) {
        self.0.set_high();
    }
}

impl<'d> BitBangUartBus<'d> {
    pub fn validate_baud(baud_rate: u32) -> Result<(), UartError> {
        if baud_rate == 0 || baud_rate as u64 > TICK_HZ / 4 {
            return Err(UartError::InvalidBaudRate);
        }
        Ok(())
    }

    pub fn new(
        tx: Peri<'d, impl embassy_rp::gpio::Pin>,
        rx: Peri<'d, impl embassy_rp::gpio::Pin>,
        baud_rate: u32,
    ) -> Result<Self, UartError> {
        Self::validate_baud(baud_rate)?;
        Ok(Self {
            tx: Output::new(tx, Level::High),
            rx: Input::new(rx, Pull::Up),
            bit_duration: Duration::from_hz(baud_rate as u64),
        })
    }
}

impl<'d> embedded_io_async::ErrorType for BitBangUartBus<'d> {
    type Error = UartError;
}

impl<'d> embedded_io_async::Read for BitBangUartBus<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, UartError> {
        let Some(byte) = buf.first_mut() else {
            return Ok(0);
        };

        self.rx.wait_for_low().await;
        Timer::after(self.bit_duration / 2).await;
        if self.rx.is_high() {
            return Err(UartError::Framing);
        }
        let mut value = 0u8;
        for i in 0..8 {
            Timer::after(self.bit_duration).await;
            if self.rx.is_high() {
                value |= 1 << i;
            }
        }
        Timer::after(self.bit_duration).await;
        if self.rx.is_low() {
            return Err(UartError::Framing);
        }
        *byte = value;
        Ok(1)
    }
}

impl<'d> embedded_io_async::Write for BitBangUartBus<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, UartError> {
        let tx = TxIdleGuard(&mut self.tx);
        for &b in buf {
            tx.0.set_low();
            Timer::after(self.bit_duration).await;
            for i in 0..8 {
                if (b & (1 << i)) != 0 {
                    tx.0.set_high();
                } else {
                    tx.0.set_low();
                }
                Timer::after(self.bit_duration).await;
            }
            tx.0.set_high();
            Timer::after(self.bit_duration).await;
        }
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), UartError> {
        Ok(())
    }
}
