use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum UartError {
    BufferFull,
    Other,
}

impl core::fmt::Display for UartError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for UartError {}

impl embedded_io_async::Error for UartError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

impl From<embassy_rp::uart::Error> for UartError {
    fn from(_: embassy_rp::uart::Error) -> Self {
        UartError::Other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum UartBusVersion {
    Hardware,
    Pio,
    BitBang,
}

pub trait DynUartBus {
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<usize, UartError>> + 'a>>;
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Pin<Box<dyn Future<Output = Result<usize, UartError>> + 'a>>;
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), UartError>> + 'a>>;
}

pub struct UartBusHandle {
    inner: Box<dyn DynUartBus>,
    version: UartBusVersion,
}

impl UartBusHandle {
    pub fn new(inner: Box<dyn DynUartBus>, version: UartBusVersion) -> Self {
        Self { inner, version }
    }

    pub fn version(&self) -> UartBusVersion {
        self.version
    }
}

impl embedded_io_async::ErrorType for UartBusHandle {
    type Error = UartError;
}

impl embedded_io_async::Read for UartBusHandle {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.read(buf).await
    }
}

impl embedded_io_async::Write for UartBusHandle {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.inner.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush().await
    }
}

impl<T> DynUartBus for T
where
    T: embedded_io_async::Read<Error = UartError> + embedded_io_async::Write<Error = UartError>,
{
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<usize, UartError>> + 'a>> {
        Box::pin(async move { embedded_io_async::Write::write(self, buf).await })
    }

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Pin<Box<dyn Future<Output = Result<usize, UartError>> + 'a>> {
        Box::pin(async move { embedded_io_async::Read::read(self, buf).await })
    }

    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), UartError>> + 'a>> {
        Box::pin(async move { embedded_io_async::Write::flush(self).await })
    }
}
