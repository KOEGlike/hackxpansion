use alloc::vec::Vec;

use crate::bus::i2c::I2cError;
use embassy_rp::Peri;
use embassy_rp::i2c::{self, Async, Config, I2c, Instance, InterruptHandler};
use embassy_rp::interrupt::typelevel::Binding;

pub struct HardwareI2cBus<'d, I: Instance> {
    i2c: I2c<'d, I, Async>,
}

impl<'d, I: Instance> HardwareI2cBus<'d, I> {
    pub fn validate_config(config: &Config) -> Result<(), I2cError> {
        if config.frequency == 0 || config.frequency > 1_000_000 {
            return Err(I2cError::Other);
        }

        let clock = embassy_rp::clocks::clk_peri_freq();
        let period = (clock + config.frequency / 2) / config.frequency;
        let low_count = period * 3 / 5;
        let high_count = period - low_count;
        if high_count < 8 || low_count < 8 || high_count > 0xffff || low_count > 0xffff {
            return Err(I2cError::Other);
        }

        let sda_hold_count = if config.frequency < 1_000_000 {
            ((clock * 3) / 10_000_000) + 1
        } else {
            if clock <= 32_000_000 {
                return Err(I2cError::Other);
            }
            ((clock * 3) / 25_000_000) + 1
        };
        if sda_hold_count > low_count - 2 {
            return Err(I2cError::Other);
        }

        Ok(())
    }

    pub fn new<Irq>(
        peri: Peri<'d, I>,
        scl: Peri<'d, impl i2c::SclPin<I> + 'd>,
        sda: Peri<'d, impl i2c::SdaPin<I> + 'd>,
        irq: Irq,
        config: Config,
    ) -> Result<Self, I2cError>
    where
        Irq: Binding<I::Interrupt, InterruptHandler<I>> + 'd,
    {
        Self::validate_config(&config)?;
        let i2c = I2c::new_async(peri, scl, sda, irq, config);
        Ok(Self { i2c })
    }
}

impl<'d, I: Instance> embedded_hal::i2c::ErrorType for HardwareI2cBus<'d, I> {
    type Error = I2cError;
}

impl<'d, I: Instance> embedded_hal_async::i2c::I2c<embedded_hal::i2c::SevenBitAddress>
    for HardwareI2cBus<'d, I>
{
    async fn read(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        if address > 0x7f {
            return Err(I2cError::AddressOutOfRange);
        }
        if read.is_empty() {
            return Err(I2cError::InvalidBufferLength);
        }
        self.i2c.read(address, read).await.map_err(I2cError::from)
    }

    async fn write(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        write: &[u8],
    ) -> Result<(), I2cError> {
        if address > 0x7f {
            return Err(I2cError::AddressOutOfRange);
        }
        if write.is_empty() {
            return Err(I2cError::InvalidBufferLength);
        }
        self.i2c.write(address, write).await.map_err(I2cError::from)
    }

    async fn write_read(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        if address > 0x7f {
            return Err(I2cError::AddressOutOfRange);
        }
        if write.is_empty() || read.is_empty() {
            return Err(I2cError::InvalidBufferLength);
        }
        self.i2c
            .write_read(address, write, read)
            .await
            .map_err(I2cError::from)
    }

    async fn transaction(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), I2cError> {
        if address > 0x7f {
            return Err(I2cError::AddressOutOfRange);
        }
        if operations.iter().any(|operation| match operation {
            embedded_hal::i2c::Operation::Read(buf) => buf.is_empty(),
            embedded_hal::i2c::Operation::Write(buf) => buf.is_empty(),
        }) {
            return Err(I2cError::InvalidBufferLength);
        }
        if operations.is_empty() {
            return Ok(());
        }

        let mut seen_read = false;
        let mut write_len = 0;
        let mut read_len = 0;
        for operation in operations.iter() {
            match operation {
                embedded_hal::i2c::Operation::Write(bytes) if !seen_read => {
                    write_len += bytes.len();
                }
                embedded_hal::i2c::Operation::Read(bytes) => {
                    seen_read = true;
                    read_len += bytes.len();
                }
                embedded_hal::i2c::Operation::Write(_) => {
                    // The public Embassy API cannot continue an atomic transaction
                    // after a read phase, so reject rather than insert a STOP.
                    return Err(I2cError::UnsupportedTransaction);
                }
            }
        }

        let mut write_buffer = Vec::with_capacity(write_len);
        for operation in operations.iter() {
            if let embedded_hal::i2c::Operation::Write(bytes) = operation {
                write_buffer.extend_from_slice(bytes);
            }
        }

        if read_len == 0 {
            return self
                .i2c
                .write(address, &write_buffer)
                .await
                .map_err(I2cError::from);
        }

        let mut read_buffer = alloc::vec![0; read_len];
        if write_len == 0 {
            self.i2c
                .read(address, &mut read_buffer)
                .await
                .map_err(I2cError::from)?;
        } else {
            self.i2c
                .write_read(address, &write_buffer, &mut read_buffer)
                .await
                .map_err(I2cError::from)?;
        }

        let mut offset = 0;
        for operation in operations.iter_mut() {
            if let embedded_hal::i2c::Operation::Read(bytes) = operation {
                let end = offset + bytes.len();
                bytes.copy_from_slice(&read_buffer[offset..end]);
                offset = end;
            }
        }
        Ok(())
    }
}
