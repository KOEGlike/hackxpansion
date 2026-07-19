use crate::bus::i2c::I2cError;
use embassy_rp::Peri;
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_time::{Duration, TICK_HZ, Timer, with_timeout};

const CLOCK_STRETCH_TIMEOUT: Duration = Duration::from_millis(25);

/// Bit-banged I2C master using open-drain GPIO for SCL/SDA.
///
/// Implements the standard I2C protocol (start, stop, ACK/NACK, read/write) by
/// bit-banging two open-drain GPIO pins. Both lines are released (high) when
/// idle, pulled low to signal, and read back to detect clock stretching and
/// NACK. Uses `Timer::after().await` for all timing so it never blocks the
/// executor.
pub struct BitBangI2cBus<'d> {
    scl: OutputOpenDrain<'d>,
    sda: OutputOpenDrain<'d>,
    half_period: Duration,
}

struct BusIdleGuard<'d> {
    scl: *mut OutputOpenDrain<'d>,
    sda: *mut OutputOpenDrain<'d>,
}

impl Drop for BusIdleGuard<'_> {
    fn drop(&mut self) {
        // SAFETY: the guard is created and dropped entirely within an operation
        // that exclusively borrows the bus. The GPIO drivers cannot move or be
        // accessed elsewhere while that operation future is alive.
        unsafe {
            (*self.sda).set_high();
            (*self.scl).set_high();
        }
    }
}

impl<'d> BitBangI2cBus<'d> {
    pub fn new(
        scl: Peri<'d, impl embassy_rp::gpio::Pin>,
        sda: Peri<'d, impl embassy_rp::gpio::Pin>,
        frequency_hz: u32,
    ) -> Result<Self, I2cError> {
        if frequency_hz == 0 || frequency_hz as u64 > TICK_HZ / 2 {
            return Err(I2cError::Other);
        }

        let mut scl = OutputOpenDrain::new(scl, Level::High);
        let mut sda = OutputOpenDrain::new(sda, Level::High);
        scl.set_pullup(true);
        sda.set_pullup(true);

        let half_period = Duration::from_hz(frequency_hz as u64 * 2);

        Ok(Self {
            scl,
            sda,
            half_period,
        })
    }

    fn release_scl(&mut self) {
        self.scl.set_high();
    }

    fn pull_scl_low(&mut self) {
        self.scl.set_low();
    }

    fn release_sda(&mut self) {
        self.sda.set_high();
    }

    fn pull_sda_low(&mut self) {
        self.sda.set_low();
    }

    fn sda_high(&self) -> bool {
        self.sda.is_high()
    }

    async fn wait_scl_high(&mut self) -> Result<(), I2cError> {
        self.release_scl();
        if self.scl.is_low()
            && with_timeout(CLOCK_STRETCH_TIMEOUT, self.scl.wait_for_high())
                .await
                .is_err()
        {
            return Err(I2cError::Abort);
        }
        Ok(())
    }

    async fn delay(&mut self) {
        Timer::after(self.half_period).await;
    }

    async fn start(&mut self) -> Result<(), I2cError> {
        self.release_sda();
        self.delay().await;
        self.wait_scl_high().await?;
        if !self.sda_high() {
            self.release_scl();
            self.release_sda();
            return Err(I2cError::Abort);
        }
        self.delay().await;
        self.pull_sda_low();
        self.delay().await;
        self.pull_scl_low();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), I2cError> {
        self.pull_sda_low();
        self.delay().await;
        let clock_result = self.wait_scl_high().await;
        if clock_result.is_ok() {
            self.delay().await;
        }
        self.release_sda();
        self.release_scl();
        self.delay().await;
        clock_result
    }

    async fn write_bit(&mut self, bit: bool) -> Result<(), I2cError> {
        if bit {
            self.release_sda();
        } else {
            self.pull_sda_low();
        }
        self.delay().await;
        self.wait_scl_high().await?;
        if bit && !self.sda_high() {
            self.release_sda();
            self.release_scl();
            return Err(I2cError::ArbitrationLoss);
        }
        self.delay().await;
        self.pull_scl_low();
        Ok(())
    }

    async fn read_bit(&mut self) -> Result<bool, I2cError> {
        self.release_sda();
        self.delay().await;
        self.wait_scl_high().await?;
        let bit = self.sda_high();
        self.delay().await;
        self.pull_scl_low();
        Ok(bit)
    }

    /// Write one byte, return true if ACK received.
    async fn write_byte(&mut self, byte: u8) -> Result<bool, I2cError> {
        for i in (0..8).rev() {
            self.write_bit((byte >> i) & 1 != 0).await?;
        }
        // ACK is low (slave pulls SDA low).
        Ok(!self.read_bit().await?)
    }

    /// Read one byte. Send ACK if `ack` is true, NACK otherwise.
    async fn read_byte(&mut self, ack: bool) -> Result<u8, I2cError> {
        let mut byte = 0u8;
        for _ in 0..8 {
            byte <<= 1;
            if self.read_bit().await? {
                byte |= 1;
            }
        }
        // Master sends ACK (low) or NACK (high).
        self.write_bit(!ack).await?;
        Ok(byte)
    }

    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), I2cError> {
        for &b in bytes {
            if !self.write_byte(b).await? {
                return Err(I2cError::Abort);
            }
        }
        Ok(())
    }

    async fn transfer(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), I2cError> {
        use embedded_hal::i2c::Operation;

        let mut read_direction = None;
        for index in 0..operations.len() {
            let is_read = matches!(&operations[index], Operation::Read(_));
            let next_is_read = matches!(operations.get(index + 1), Some(Operation::Read(_)));

            if read_direction != Some(is_read) {
                if read_direction.is_some() {
                    self.start().await?;
                }
                if !self.write_byte((address << 1) | u8::from(is_read)).await? {
                    return Err(I2cError::Abort);
                }
                read_direction = Some(is_read);
            }

            match &mut operations[index] {
                Operation::Read(buf) => {
                    let len = buf.len();
                    for (byte_index, byte) in buf.iter_mut().enumerate() {
                        let ack = byte_index + 1 < len || next_is_read;
                        *byte = self.read_byte(ack).await?;
                    }
                }
                Operation::Write(buf) => self.write_bytes(buf).await?,
            }
        }
        Ok(())
    }

    async fn transaction_impl(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), I2cError> {
        use embedded_hal::i2c::Operation;

        if address > 0x7f {
            return Err(I2cError::AddressOutOfRange);
        }
        if operations.iter().any(|operation| match operation {
            Operation::Read(buf) => buf.is_empty(),
            Operation::Write(buf) => buf.is_empty(),
        }) {
            return Err(I2cError::InvalidBufferLength);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let _idle_guard = BusIdleGuard {
            scl: &mut self.scl,
            sda: &mut self.sda,
        };
        self.start().await?;
        let transfer_result = self.transfer(address, operations).await;
        if transfer_result == Err(I2cError::ArbitrationLoss) {
            self.release_sda();
            self.release_scl();
            return transfer_result;
        }
        let stop_result = self.stop().await;
        transfer_result.and(stop_result)
    }
}

impl<'d> embedded_hal::i2c::ErrorType for BitBangI2cBus<'d> {
    type Error = I2cError;
}

impl<'d> embedded_hal_async::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for BitBangI2cBus<'d> {
    async fn read(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        self.transaction_impl(address, &mut [embedded_hal::i2c::Operation::Read(read)])
            .await
    }

    async fn write(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        write: &[u8],
    ) -> Result<(), I2cError> {
        self.transaction_impl(address, &mut [embedded_hal::i2c::Operation::Write(write)])
            .await
    }

    async fn write_read(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        self.transaction_impl(
            address,
            &mut [
                embedded_hal::i2c::Operation::Write(write),
                embedded_hal::i2c::Operation::Read(read),
            ],
        )
        .await
    }

    async fn transaction(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), I2cError> {
        self.transaction_impl(address, operations).await
    }
}
