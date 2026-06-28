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

impl embedded_hal::spi::Error for SpiError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            SpiError::Overrun => embedded_hal::spi::ErrorKind::Overrun,
            SpiError::ModeFault => embedded_hal::spi::ErrorKind::ModeFault,
            SpiError::Crc => embedded_hal::spi::ErrorKind::FrameFormat,
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

    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), SpiError> {
        self.inner.transfer_in_place(words).await
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

// ── embedded-hal trait impls on SpiBusHandle ──────────────────────────

impl embedded_hal::spi::ErrorType for SpiBusHandle {
    type Error = SpiError;
}

impl embedded_hal::spi::SpiBus<u8> for SpiBusHandle {
    fn flush(&mut self) -> Result<(), SpiError> {
        Ok(())
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
        let len = words.len();
        for i in 0..len {
            let w = words[i];
            self.transfer_blocking(&mut words[i..i + 1], &[w])?;
        }
        Ok(())
    }
}

impl embedded_hal_async::spi::SpiBus<u8> for SpiBusHandle {
    async fn flush(&mut self) -> Result<(), SpiError> {
        Ok(())
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

impl embedded_hal_async::spi::SpiDevice<u8> for SpiBusHandle {
    async fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), SpiError> {
        for op in operations {
            match op {
                embedded_hal::spi::Operation::Read(buf) => {
                    self.inner.read(buf).await?;
                }
                embedded_hal::spi::Operation::Write(buf) => {
                    self.inner.write(buf).await?;
                }
                embedded_hal::spi::Operation::Transfer(read, write) => {
                    self.inner.transfer(read, write).await?;
                }
                embedded_hal::spi::Operation::TransferInPlace(buf) => {
                    self.inner.transfer_in_place(buf).await?;
                }
                embedded_hal::spi::Operation::DelayNs(_) => {}
            }
        }
        Ok(())
    }
}

// Blanket impls: anything implementing embedded-hal SpiBus<u8> gets our traits for free

impl<T> DynSpiBusBlocking for T
where
    T: embedded_hal::spi::SpiBus<u8>,
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
