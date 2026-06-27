use crate::bus::i2c::I2cError;
use embassy_rp::Peri;
use embassy_rp::gpio::{Level, OutputOpenDrain};
use embassy_time::{Duration, Timer};

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

impl<'d> BitBangI2cBus<'d> {
    pub fn new(
        scl: Peri<'d, impl embassy_rp::gpio::Pin>,
        sda: Peri<'d, impl embassy_rp::gpio::Pin>,
        frequency_hz: u32,
    ) -> Self {
        let mut scl = OutputOpenDrain::new(scl, Level::High);
        let mut sda = OutputOpenDrain::new(sda, Level::High);
        scl.set_pullup(true);
        sda.set_pullup(true);

        let half_period = if frequency_hz > 0 {
            Duration::from_micros(500_000 / frequency_hz as u64)
        } else {
            Duration::from_micros(5)
        };

        Self {
            scl,
            sda,
            half_period,
        }
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

    async fn wait_scl_high(&mut self) {
        // Clock stretching: wait for the slave to release SCL.
        loop {
            self.release_scl();
            Timer::after(self.half_period).await;
            if self.scl.is_high() {
                return;
            }
        }
    }

    async fn delay(&mut self) {
        Timer::after(self.half_period).await;
    }

    async fn start(&mut self) {
        // From idle (both high), SDA falls while SCL is high.
        self.release_sda();
        self.release_scl();
        self.delay().await;
        self.pull_sda_low();
        self.delay().await;
        self.pull_scl_low();
        self.delay().await;
    }

    async fn stop(&mut self) {
        // SDA rises while SCL is high.
        self.pull_sda_low();
        self.delay().await;
        self.wait_scl_high().await;
        self.release_sda();
        self.delay().await;
    }

    async fn write_bit(&mut self, bit: bool) {
        if bit {
            self.release_sda();
        } else {
            self.pull_sda_low();
        }
        self.delay().await;
        self.wait_scl_high().await;
        self.delay().await;
        self.pull_scl_low();
    }

    async fn read_bit(&mut self) -> bool {
        self.release_sda();
        self.delay().await;
        self.wait_scl_high().await;
        let bit = self.sda_high();
        self.delay().await;
        self.pull_scl_low();
        bit
    }

    /// Write one byte, return true if ACK received.
    async fn write_byte(&mut self, byte: u8) -> bool {
        for i in (0..8).rev() {
            self.write_bit((byte >> i) & 1 != 0).await;
        }
        // ACK is low (slave pulls SDA low).
        !self.read_bit().await
    }

    /// Read one byte. Send ACK if `ack` is true, NACK otherwise.
    async fn read_byte(&mut self, ack: bool) -> u8 {
        let mut byte = 0u8;
        for _ in 0..8 {
            byte <<= 1;
            if self.read_bit().await {
                byte |= 1;
            }
        }
        // Master sends ACK (low) or NACK (high).
        self.write_bit(!ack).await;
        byte
    }

    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), I2cError> {
        for &b in bytes {
            if !self.write_byte(b).await {
                return Err(I2cError::Abort);
            }
        }
        Ok(())
    }

    async fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), I2cError> {
        let len = buf.len();
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = self.read_byte(i < len - 1).await;
        }
        Ok(())
    }
}

impl<'d> embedded_hal_1::i2c::ErrorType for BitBangI2cBus<'d> {
    type Error = I2cError;
}

impl<'d> embedded_hal_async::i2c::I2c<embedded_hal_1::i2c::SevenBitAddress> for BitBangI2cBus<'d> {
    async fn read(
        &mut self,
        address: embedded_hal_1::i2c::SevenBitAddress,
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        if read.is_empty() {
            return Err(I2cError::InvalidBufferLength);
        }
        self.start().await;
        // Address + R bit
        if !self.write_byte((address << 1) | 1).await {
            self.stop().await;
            return Err(I2cError::Abort);
        }
        self.read_bytes(read).await?;
        self.stop().await;
        Ok(())
    }

    async fn write(
        &mut self,
        address: embedded_hal_1::i2c::SevenBitAddress,
        write: &[u8],
    ) -> Result<(), I2cError> {
        if write.is_empty() {
            return Err(I2cError::InvalidBufferLength);
        }
        self.start().await;
        if !self.write_byte(address << 1).await {
            self.stop().await;
            return Err(I2cError::Abort);
        }
        self.write_bytes(write).await?;
        self.stop().await;
        Ok(())
    }

    async fn write_read(
        &mut self,
        address: embedded_hal_1::i2c::SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        if write.is_empty() || read.is_empty() {
            return Err(I2cError::InvalidBufferLength);
        }
        self.start().await;
        // Write phase
        if !self.write_byte(address << 1).await {
            self.stop().await;
            return Err(I2cError::Abort);
        }
        self.write_bytes(write).await?;
        // Repeated start for read phase
        self.start().await;
        if !self.write_byte((address << 1) | 1).await {
            self.stop().await;
            return Err(I2cError::Abort);
        }
        self.read_bytes(read).await?;
        self.stop().await;
        Ok(())
    }

    async fn transaction(
        &mut self,
        address: embedded_hal_1::i2c::SevenBitAddress,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), I2cError> {
        use embedded_hal_1::i2c::Operation;

        if operations.is_empty() {
            return Ok(());
        }

        self.start().await;

        let len = operations.len();
        let mut first = true;
        let mut last_op_was_read = false;

        for (i, op) in operations.iter_mut().enumerate() {
            let is_read = matches!(op, Operation::Read(_));
            let last = i == len - 1;

            // (Re)start with the correct R/W bit.
            if !first || last_op_was_read != is_read {
                self.start().await;
            }
            first = false;

            let addr_byte = (address << 1) | u8::from(is_read);
            if !self.write_byte(addr_byte).await {
                self.stop().await;
                return Err(I2cError::Abort);
            }

            match op {
                Operation::Read(buf) => {
                    if buf.is_empty() {
                        self.stop().await;
                        return Err(I2cError::InvalidBufferLength);
                    }
                    let len = buf.len();
                    for (j, byte) in buf.iter_mut().enumerate() {
                        *byte = self.read_byte(j < len - 1).await;
                    }
                }
                Operation::Write(buf) => {
                    if buf.is_empty() {
                        self.stop().await;
                        return Err(I2cError::InvalidBufferLength);
                    }
                    self.write_bytes(buf).await?;
                }
            }

            last_op_was_read = is_read;

            if last {
                self.stop().await;
            }
        }

        Ok(())
    }
}
