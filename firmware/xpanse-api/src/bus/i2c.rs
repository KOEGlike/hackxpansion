//! I2C bus abstraction.
//!
//! Wraps a concrete backend (hardware, PIO, or bit-banged) behind a single
//! boxed trait object so that drivers can operate on an `I2cBusHandle`
//! without knowing which backend was selected.
//!
//! [`BusAllocator`](crate::bus::allocator::BusAllocator) is the entry point used to
//! allocate I2C buses at startup.

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use embedded_hal::i2c::{ErrorKind, Operation, SevenBitAddress};

/// Error returned by I2C operations.
///
/// # Example
///
/// ```ignore
/// use xpanse_api::bus::i2c::{I2cBusHandle, I2cError};
///
/// async fn read_register(bus: &mut I2cBusHandle, dev: u8, reg: u8) -> Result<u8, I2cError> {
///     let mut buf = [0u8; 1];
///     bus.write_read(dev, &[reg], &mut buf).await?;
///     Ok(buf[0])
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum I2cError {
    Abort,
    ArbitrationLoss,
    InvalidBufferLength,
    AddressOutOfRange,
    UnsupportedTransaction,
    /// Any other error reported by the underlying backend.
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
            I2cError::ArbitrationLoss => ErrorKind::ArbitrationLoss,
            I2cError::InvalidBufferLength => ErrorKind::Other,
            I2cError::AddressOutOfRange => ErrorKind::Other,
            I2cError::UnsupportedTransaction => ErrorKind::Other,
            I2cError::Other => ErrorKind::Other,
        }
    }
}

impl From<embassy_rp::i2c::Error> for I2cError {
    fn from(e: embassy_rp::i2c::Error) -> Self {
        match e {
            embassy_rp::i2c::Error::Abort(embassy_rp::i2c::AbortReason::ArbitrationLoss) => {
                I2cError::ArbitrationLoss
            }
            embassy_rp::i2c::Error::Abort(_) => I2cError::Abort,
            embassy_rp::i2c::Error::InvalidReadBufferLength
            | embassy_rp::i2c::Error::InvalidWriteBufferLength => I2cError::InvalidBufferLength,
            embassy_rp::i2c::Error::AddressOutOfRange(_) => I2cError::AddressOutOfRange,
            _ => I2cError::Other,
        }
    }
}

/// Backend variant of an [`I2cBusHandle`].
/// No PIO yet
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum I2cBusVersion {
    /// Hardware I2C peripheral.
    Hardware,
    /// Bit-banged via GPIO.
    BitBang,
    // PIO variant reserved for when embassy adds a PIO I2C program.
}

/// Trait-object-safe I2C bus operating on 7-bit addresses.
pub trait DynI2cBus: Send {
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

    fn transaction<'a, 'op>(
        &'a mut self,
        address: u8,
        operations: &'a mut [Operation<'op>],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>>
    where
        'op: 'a;
}

/// Owned handle to an async I2C bus.
///
/// Dropping this handle does not return its startup resources, so keep it alive
/// for as long as you need I2C access.
#[must_use = "dropping a bus handle does not return its startup resources"]
pub struct I2cBusHandle {
    inner: Box<dyn DynI2cBus>,
    version: I2cBusVersion,
}

impl I2cBusHandle {
    /// Wraps a boxed backend and records its [`I2cBusVersion`].
    pub fn new(inner: Box<dyn DynI2cBus>, version: I2cBusVersion) -> Self {
        Self { inner, version }
    }

    /// Returns the backend variant.
    pub fn version(&self) -> I2cBusVersion {
        self.version
    }

    /// Read bytes from a 7-bit address.
    pub async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), I2cError> {
        self.inner.read(address, read).await
    }

    /// Write bytes to a 7-bit address.
    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), I2cError> {
        self.inner.write(address, write).await
    }

    /// Write the `write` slice, then read into the `read` slice without releasing the bus.
    pub async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), I2cError> {
        self.inner.write_read(address, write, read).await
    }

    /// Run a sequence of read/write operations atomically on the bus.
    pub async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), I2cError> {
        self.inner.transaction(address, operations).await
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
        self.inner.transaction(address, operations).await
    }
}

// ── blanket impl: anything that impls embedded-hal-async I2c<SevenBitAddress>
//    with a compatible error gets DynI2cBus for free. ──

impl<T> DynI2cBus for T
where
    T: embedded_hal_async::i2c::I2c<SevenBitAddress>,
    T::Error: Into<I2cError>,
    T: Send,
{
    fn read<'a>(
        &'a mut self,
        address: u8,
        read: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>> {
        Box::pin(async move {
            if address > 0x7f {
                return Err(I2cError::AddressOutOfRange);
            }
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
            if address > 0x7f {
                return Err(I2cError::AddressOutOfRange);
            }
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
            if address > 0x7f {
                return Err(I2cError::AddressOutOfRange);
            }
            embedded_hal_async::i2c::I2c::write_read(self, address, write, read)
                .await
                .map_err(Into::into)
        })
    }

    fn transaction<'a, 'op>(
        &'a mut self,
        address: u8,
        operations: &'a mut [Operation<'op>],
    ) -> Pin<Box<dyn Future<Output = Result<(), I2cError>> + 'a>>
    where
        'op: 'a,
    {
        Box::pin(async move {
            if address > 0x7f {
                return Err(I2cError::AddressOutOfRange);
            }
            embedded_hal_async::i2c::I2c::transaction(self, address, operations)
                .await
                .map_err(Into::into)
        })
    }
}
