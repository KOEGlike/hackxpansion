use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use embedded_hal::i2c::{ErrorKind, Operation, SevenBitAddress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum I2cError {
    /// I2C abort — e.g. NACK, bus stuck.
    Abort,
    /// Invalid buffer length (zero-length read or write).
    InvalidBufferLength,
    /// Address out of range.
    AddressOutOfRange,
    Other,
}

impl core::fmt::Display for I2cError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for I2cError {}

impl embedded_hal::i2c::Error for I2cError {
    fn kind(&self) -> ErrorKind {
        match self {
            I2cError::Abort => {
                ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Unknown)
            }
            I2cError::InvalidBufferLength => ErrorKind::Other,
            I2cError::AddressOutOfRange => ErrorKind::Other,
            I2cError::Other => ErrorKind::Other,
        }
    }
}

impl From<embassy_rp::i2c::Error> for I2cError {
    fn from(e: embassy_rp::i2c::Error) -> Self {
        match e {
            embassy_rp::i2c::Error::Abort(_) => I2cError::Abort,
            embassy_rp::i2c::Error::InvalidReadBufferLength
            | embassy_rp::i2c::Error::InvalidWriteBufferLength => I2cError::InvalidBufferLength,
            embassy_rp::i2c::Error::AddressOutOfRange(_) => I2cError::AddressOutOfRange,
            _ => I2cError::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum I2cBusVersion {
    Hardware,
    BitBang,
    // PIO variant reserved for when embassy adds a PIO I2C program.
}

/// Trait-object-safe I2C bus operating on 7-bit addresses. Exposes the three
/// primitive I2C operations (`read`, `write`, `write_read`) with boxed futures
/// so a single [`I2cBusHandle`] can hide the concrete backend.
///
/// `transaction` is **not** part of this trait because its lifetime structure
/// (a slice of `Operation<'_>` with independent lifetimes) is incompatible with
/// trait-object dispatch. The handle implements `embedded_hal_async::i2c::I2c`
/// by decomposing transactions into these primitives.
pub trait DynI2cBus {
    fn read<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>>;

    fn write<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>>;

    fn write_read<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
        read: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>>;
}

pub struct I2cBusHandle {
    inner: Box<dyn DynI2cBus>,
    version: I2cBusVersion,
}

impl I2cBusHandle {
    pub fn new(inner: Box<dyn DynI2cBus>, version: I2cBusVersion) -> Self {
        Self { inner, version }
    }

    pub fn version(&self) -> I2cBusVersion {
        self.version
    }

    pub async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), I2cError> {
        self.inner.read(address, read).await
    }

    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), I2cError> {
        self.inner.write(address, write).await
    }

    pub async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        self.inner.write_read(address, write, read).await
    }
}

impl embedded_hal::i2c::ErrorType for I2cBusHandle {
    type Error = I2cError;
}

impl embedded_hal_async::i2c::I2c<SevenBitAddress> for I2cBusHandle {
    async fn read(&mut self, address: SevenBitAddress, read: &mut [u8]) -> Result<(), I2cError> {
        self.inner.read(address, read).await
    }

    async fn write(&mut self, address: SevenBitAddress, write: &[u8]) -> Result<(), I2cError> {
        self.inner.write(address, write).await
    }

    async fn write_read(
        &mut self,
        address: SevenBitAddress,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        self.inner.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), I2cError> {
        for op in operations {
            match op {
                Operation::Read(buf) => self.inner.read(address, buf).await?,
                Operation::Write(buf) => self.inner.write(address, buf).await?,
            }
        }
        Ok(())
    }
}

// ── blanket impl: anything that impls embedded-hal-async I2c<SevenBitAddress>
//    with a compatible error gets DynI2cBus for free. ──

impl<T> DynI2cBus for T
where
    T: embedded_hal_async::i2c::I2c<SevenBitAddress>,
    T::Error: Into<I2cError>,
{
    fn read<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::i2c::I2c::read(self, address, read)
                .await
                .map_err(Into::into)
        })
    }

    fn write<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::i2c::I2c::write(self, address, write)
                .await
                .map_err(Into::into)
        })
    }

    fn write_read<'a>(
        &'a mut self,
        address: u8,
        write: &'a [u8],
        read: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::i2c::I2c::write_read(self, address, write, read)
                .await
                .map_err(Into::into)
        })
    }
}
