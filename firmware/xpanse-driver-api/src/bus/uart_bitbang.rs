use crate::bus::uart::UartError;
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::{Duration, Timer};

pub struct BitBangUartBus<'d> {
    tx: Output<'d>,
    rx: Input<'d>,
    bit_duration: Duration,
}

impl<'d> BitBangUartBus<'d> {
    pub fn new(
        tx: Peri<'d, impl embassy_rp::gpio::Pin>,
        rx: Peri<'d, impl embassy_rp::gpio::Pin>,
        baud_rate: u32,
    ) -> Self {
        Self {
            tx: Output::new(tx, Level::High),
            rx: Input::new(rx, Pull::Up),
            bit_duration: Duration::from_micros(1_000_000 / baud_rate as u64),
        }
    }
}

impl<'d> embedded_io_async::ErrorType for BitBangUartBus<'d> {
    type Error = UartError;
}

impl<'d> embedded_io_async::Read for BitBangUartBus<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, UartError> {
        for b in buf.iter_mut() {
            self.rx.wait_for_low().await;
            Timer::after(self.bit_duration / 2).await;
            let mut val = 0u8;
            for i in 0..8 {
                Timer::after(self.bit_duration).await;
                if self.rx.is_high() {
                    val |= 1 << i;
                }
            }
            Timer::after(self.bit_duration).await;
            *b = val;
        }
        Ok(buf.len())
    }
}

impl<'d> embedded_io_async::Write for BitBangUartBus<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, UartError> {
        for &b in buf {
            self.tx.set_low();
            Timer::after(self.bit_duration).await;
            for i in 0..8 {
                if (b & (1 << i)) != 0 {
                    self.tx.set_high();
                } else {
                    self.tx.set_low();
                }
                Timer::after(self.bit_duration).await;
            }
            self.tx.set_high();
            Timer::after(self.bit_duration).await;
        }
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), UartError> {
        Ok(())
    }
}
