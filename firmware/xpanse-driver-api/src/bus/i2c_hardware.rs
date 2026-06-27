use crate::bus::i2c::I2cError;
use embassy_rp::Peri;
use embassy_rp::i2c::{self, Async, Config, I2c, Instance, InterruptHandler};
use embassy_rp::interrupt::typelevel::Binding;

pub struct HardwareI2cBus<'d, I: Instance> {
    i2c: I2c<'d, I, Async>,
}

impl<'d, I: Instance> HardwareI2cBus<'d, I> {
    pub fn new<Irq>(
        peri: Peri<'d, I>,
        scl: Peri<'d, impl i2c::SclPin<I> + 'd>,
        sda: Peri<'d, impl i2c::SdaPin<I> + 'd>,
        irq: Irq,
        config: Config,
    ) -> Self
    where
        Irq: Binding<I::Interrupt, InterruptHandler<I>> + 'd,
    {
        let i2c = I2c::new_async(peri, scl, sda, irq, config);
        Self { i2c }
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
        self.i2c.read(address, read).await.map_err(I2cError::from)
    }

    async fn write(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        write: &[u8],
    ) -> Result<(), I2cError> {
        self.i2c.write(address, write).await.map_err(I2cError::from)
    }

    async fn write_read(
        &mut self,
        address: embedded_hal::i2c::SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
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
        self.i2c
            .transaction(address, operations)
            .await
            .map_err(I2cError::from)
    }
}
