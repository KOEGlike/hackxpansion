use embassy_rp::peripherals::{I2C0, I2C1, SPI1, UART0, UART1};
use embassy_rp::spi;
use embassy_rp::Peri;

pub enum SpiResource {
    HwSpi1(Peri<'static, SPI1>),
    Pio { block: u8, sm: u8 },
    BitBang,
}

pub struct BusAllocator {
    spi1_peri: Option<Peri<'static, SPI1>>,
    pio_sm_used: u16,
    _i2c0: Option<Peri<'static, I2C0>>,
    _i2c1: Option<Peri<'static, I2C1>>,
    _uart0: Option<Peri<'static, UART0>>,
    _uart1: Option<Peri<'static, UART1>>,
}

impl BusAllocator {
    pub fn new(
        spi1: Peri<'static, SPI1>,
        i2c0: Peri<'static, I2C0>,
        i2c1: Peri<'static, I2C1>,
        uart0: Peri<'static, UART0>,
        uart1: Peri<'static, UART1>,
    ) -> Self {
        Self {
            spi1_peri: Some(spi1),
            pio_sm_used: 0,
            _i2c0: Some(i2c0),
            _i2c1: Some(i2c1),
            _uart0: Some(uart0),
            _uart1: Some(uart1),
        }
    }

    pub fn request_spi<I: spi::Instance + 'static>(&mut self) -> SpiResource {
        use core::any::TypeId;

        if TypeId::of::<I>() == TypeId::of::<SPI1>() {
            if let Some(peri) = self.spi1_peri.take() {
                return SpiResource::HwSpi1(peri);
            }
        }

        for block in 0..3u8 {
            for sm in 0..4u8 {
                let bit = block * 4 + sm;
                if self.pio_sm_used & (1 << bit) == 0 {
                    self.pio_sm_used |= 1 << bit;
                    return SpiResource::Pio { block, sm };
                }
            }
        }

        SpiResource::BitBang
    }

    pub fn request_spi_typed<I: spi::Instance + 'static>(&mut self) -> (super::spi::SpiBusVersion, SpiResource) {
        let resource = self.request_spi::<I>();
        let version = match &resource {
            SpiResource::HwSpi1(_) => super::spi::SpiBusVersion::Hardware,
            SpiResource::Pio { .. } => super::spi::SpiBusVersion::Pio,
            SpiResource::BitBang => super::spi::SpiBusVersion::BitBang,
        };
        (version, resource)
    }

    pub fn release_pio_sm(&mut self, block: u8, sm: u8) {
        let bit = block * 4 + sm;
        self.pio_sm_used &= !(1 << bit);
    }
}
