use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SpiError {
    Overrun,
    ModeFault,
    Crc,
    InvalidFrequency,
    Other,
}

impl embedded_hal::spi::Error for SpiError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            SpiError::Overrun => embedded_hal::spi::ErrorKind::Overrun,
            SpiError::ModeFault => embedded_hal::spi::ErrorKind::ModeFault,
            SpiError::Crc => embedded_hal::spi::ErrorKind::FrameFormat,
            SpiError::InvalidFrequency => embedded_hal::spi::ErrorKind::Other,
            SpiError::Other => embedded_hal::spi::ErrorKind::Other,
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
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>>;

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

    fn transfer_in_place<'a>(
        &'a mut self,
        words: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>>;
}

pub trait DynSpiBusBlocking {
    fn flush_blocking(&mut self) -> Result<(), SpiError>;
    fn write_blocking(&mut self, data: &[u8]) -> Result<(), SpiError>;
    fn read_blocking(&mut self, data: &mut [u8]) -> Result<(), SpiError>;
    fn transfer_blocking(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError>;
    fn transfer_in_place_blocking(&mut self, words: &mut [u8]) -> Result<(), SpiError>;
}

pub trait DynSpiBusCombined: DynSpiBus + DynSpiBusBlocking + Send {}
impl<T: DynSpiBus + DynSpiBusBlocking + Send> DynSpiBusCombined for T {}

#[must_use = "dropping a bus handle does not return its startup resources"]
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

    pub async fn flush(&mut self) -> Result<(), SpiError> {
        self.inner.flush().await
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

    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.inner.transfer_in_place(words).await
    }

    pub fn flush_blocking(&mut self) -> Result<(), SpiError> {
        self.inner.flush_blocking()
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

    pub fn transfer_in_place_blocking(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.inner.transfer_in_place_blocking(words)
    }
}

// ── embedded-hal trait impls on SpiBusHandle ──────────────────────────

impl embedded_hal::spi::ErrorType for SpiBusHandle {
    type Error = SpiError;
}

impl embedded_hal::spi::SpiBus<u8> for SpiBusHandle {
    fn flush(&mut self) -> Result<(), SpiError> {
        self.flush_blocking()
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.read_blocking(words)
    }

    fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        self.write_blocking(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.transfer_blocking(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.transfer_in_place_blocking(words)
    }
}

impl embedded_hal_async::spi::SpiBus<u8> for SpiBusHandle {
    async fn flush(&mut self) -> Result<(), SpiError> {
        self.flush().await
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.read(words).await
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        self.write(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.inner.transfer_in_place(words).await
    }
}

// Blanket impls: anything implementing embedded-hal SpiBus<u8> gets our traits for free

impl<T> DynSpiBusBlocking for T
where
    T: embedded_hal::spi::SpiBus<u8>,
    T::Error: Into<SpiError>,
    T: Send,
{
    fn flush_blocking(&mut self) -> Result<(), SpiError> {
        self.flush().map_err(|e| e.into())
    }

    fn write_blocking(&mut self, data: &[u8]) -> Result<(), SpiError> {
        self.write(data).map_err(|e| e.into())
    }

    fn read_blocking(&mut self, data: &mut [u8]) -> Result<(), SpiError> {
        self.read(data).map_err(|e| e.into())
    }

    fn transfer_blocking(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        self.transfer(read, write).map_err(|e| e.into())
    }

    fn transfer_in_place_blocking(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.transfer_in_place(words).map_err(|e| e.into())
    }
}

impl<T> DynSpiBus for T
where
    T: embedded_hal_async::spi::SpiBus<u8>,
    T::Error: Into<SpiError>,
    T: Send,
{
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::spi::SpiBus::flush(self)
                .await
                .map_err(|e| e.into())
        })
    }

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

    fn transfer_in_place<'a>(
        &'a mut self,
        words: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SpiError>> + 'a>> {
        Box::pin(async move {
            embedded_hal_async::spi::SpiBus::transfer_in_place(self, words)
                .await
                .map_err(|e| e.into())
        })
    }
}
