use embassy_rp::peripherals::{I2C0, I2C1, PIO0, PIO1, PIO2, SPI1, UART0, UART1};
use embassy_rp::pio::PioPin;
use embassy_rp::spi::{self, ClkPin, MisoPin, MosiPin};
use embassy_rp::{i2c, Peri};

use crate::bus::spi_factory::PioManager;
use crate::bus::spi::{SpiBusHandle, SpiBusVersion};

pub enum SpiResource {
    HwSpi1(Peri<'static, SPI1>),
    Pio { block: u8, sm: u8 },
    BitBang,
}

pub enum I2cResource {
    HwI2c0(Peri<'static, I2C0>),
    HwI2c1(Peri<'static, I2C1>),
    BitBang,
}

pub struct BusAllocator {
    spi1_peri: Option<Peri<'static, SPI1>>,
    i2c0_peri: Option<Peri<'static, I2C0>>,
    i2c1_peri: Option<Peri<'static, I2C1>>,
    pio_sm_used: u16,
    pio_manager: PioManager,
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
        pio0: Peri<'static, PIO0>,
        pio1: Peri<'static, PIO1>,
        pio2: Peri<'static, PIO2>,
    ) -> Self {
        Self {
            spi1_peri: Some(spi1),
            i2c0_peri: Some(i2c0),
            i2c1_peri: Some(i2c1),
            pio_sm_used: 0,
            pio_manager: PioManager::new(pio0, pio1, pio2),
            _uart0: Some(uart0),
            _uart1: Some(uart1),
        }
    }

    // ── SPI ──────────────────────────────────────────────────────────

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

    pub fn request_spi_typed<I: spi::Instance + 'static>(&mut self) -> (SpiBusVersion, SpiResource) {
        let resource = self.request_spi::<I>();
        let version = match &resource {
            SpiResource::HwSpi1(_) => SpiBusVersion::Hardware,
            SpiResource::Pio { .. } => SpiBusVersion::Pio,
            SpiResource::BitBang => SpiBusVersion::BitBang,
        };
        (version, resource)
    }

    pub fn release_pio_sm(&mut self, block: u8, sm: u8) {
        let bit = block * 4 + sm;
        self.pio_sm_used &= !(1 << bit);
    }

    /// Request SPI and build a handle — PIO/bit‑bang fallback (works with any bank).
    pub fn create_spi_bus<I: spi::Instance + 'static>(
        &mut self,
        clk: Peri<'static, impl PioPin>,
        mosi: Peri<'static, impl PioPin>,
        miso: Peri<'static, impl PioPin>,
        config: spi::Config,
    ) -> SpiBusHandle {
        let resource = self.request_spi::<I>();
        crate::bus::spi_factory::create_spi_bus_pio(resource, clk, mosi, miso, config, &mut self.pio_manager)
    }

    /// Request SPI and build a handle — hardware SPI1 + fallback.
    /// Only works when pins satisfy `ClkPin<SPI1>` / `MosiPin<SPI1>` / `MisoPin<SPI1>`.
    pub fn create_spi_bus_hw<I: spi::Instance + 'static>(
        &mut self,
        clk: Peri<'static, impl PioPin + ClkPin<SPI1>>,
        mosi: Peri<'static, impl PioPin + MosiPin<SPI1>>,
        miso: Peri<'static, impl PioPin + MisoPin<SPI1>>,
        config: spi::Config,
    ) -> SpiBusHandle {
        let resource = self.request_spi::<I>();
        crate::bus::spi_factory::create_spi_bus(resource, clk, mosi, miso, config, &mut self.pio_manager)
    }

    // ── I2C ──────────────────────────────────────────────────────────

    pub fn request_i2c<I: i2c::Instance + 'static>(&mut self) -> I2cResource {
        use core::any::TypeId;

        if TypeId::of::<I>() == TypeId::of::<I2C0>() {
            if let Some(peri) = self.i2c0_peri.take() {
                return I2cResource::HwI2c0(peri);
            }
        }
        if TypeId::of::<I>() == TypeId::of::<I2C1>() {
            if let Some(peri) = self.i2c1_peri.take() {
                return I2cResource::HwI2c1(peri);
            }
        }

        I2cResource::BitBang
    }
}
