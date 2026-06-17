use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SpiError {
    Overrun,
    ModeFault,
    Crc,
    Other,
}

impl embedded_hal_1::spi::Error for SpiError {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match self {
            SpiError::Overrun => embedded_hal_1::spi::ErrorKind::Overrun,
            SpiError::ModeFault => embedded_hal_1::spi::ErrorKind::ModeFault,
            SpiError::Crc => embedded_hal_1::spi::ErrorKind::FrameFormat,
            SpiError::Other => embedded_hal_1::spi::ErrorKind::Other,
        }
    }
}

impl From<embassy_rp::spi::Error> for SpiError {
    fn from(_: embassy_rp::spi::Error) -> Self {
        SpiError::Other
    }
}

impl From<embassy_rp::pio_programs::spi::Error> for SpiError {
    fn from(_: embassy_rp::pio_programs::spi::Error) -> Self {
        SpiError::Other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SpiBusVersion {
    Hardware,
    Pio,
    BitBang,
}

pub trait DynSpiBus {
    fn write<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>>;

    fn read<'a>(
        &'a mut self,
        data: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>>;

    fn transfer<'a>(
        &'a mut self,
        read: &'a mut [u8],
        write: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>>;
}

pub trait DynSpiBusBlocking {
    fn write_blocking(&mut self, data: &[u8]) -> Result<(), SpiError>;
    fn read_blocking(&mut self, data: &mut [u8]) -> Result<(), SpiError>;
    fn transfer_blocking(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError>;
}

pub trait DynSpiBusCombined: DynSpiBus + DynSpiBusBlocking {}
impl<T: DynSpiBus + DynSpiBusBlocking> DynSpiBusCombined for T {}

pub struct SpiBusHandle {
    inner: Box<dyn DynSpiBusCombined>,
    version: SpiBusVersion,
}

impl SpiBusHandle {
    pub fn new(inner: Box<dyn DynSpiBusCombined>, version: SpiBusVersion) -> Self {
        Self { inner, version }
    }

    pub fn version(&self) -> SpiBusVersion {
        self.version
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), SpiError> {
        self.inner.write(data).await
    }

    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), SpiError> {
        self.inner.read(data).await
    }

    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.inner.transfer(read, write).await
    }

    pub fn write_blocking(&mut self, data: &[u8]) -> Result<(), SpiError> {
        self.inner.write_blocking(data)
    }

    pub fn read_blocking(&mut self, data: &mut [u8]) -> Result<(), SpiError> {
        self.inner.read_blocking(data)
    }

    pub fn transfer_blocking(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.inner.transfer_blocking(read, write)
    }
}

// Blanket impls: anything implementing embedded-hal SpiBus<u8> gets our traits for free

impl<T> DynSpiBusBlocking for T
where
    T: embedded_hal_1::spi::SpiBus<u8>,
    T::Error: Into<SpiError>,
{
    fn write_blocking(&mut self, data: &[u8]) -> Result<(), SpiError> {
        self.write(data).map_err(|e| e.into())
    }

    fn read_blocking(&mut self, data: &mut [u8]) -> Result<(), SpiError> {
        self.read(data).map_err(|e| e.into())
    }

    fn transfer_blocking(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.transfer(read, write).map_err(|e| e.into())
    }
}

impl<T> DynSpiBus for T
where
    T: embedded_hal_async::spi::SpiBus<u8>,
    T::Error: Into<SpiError>,
{
    fn write<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::spi::SpiBus::write(self, data)
                .await
                .map_err(|e| e.into())
        })
    }

    fn read<'a>(
        &'a mut self,
        data: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::spi::SpiBus::read(self, data)
                .await
                .map_err(|e| e.into())
        })
    }

    fn transfer<'a>(
        &'a mut self,
        read: &'a mut [u8],
        write: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::spi::SpiBus::transfer(self, read, write)
                .await
                .map_err(|e| e.into())
        })
    }
}
