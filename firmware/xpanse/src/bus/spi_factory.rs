use alloc::boxed::Box;

use embassy_rp::peripherals::{PIO0, PIO1, PIO2};
use embassy_rp::pio::{Pio, PioPin};
use embassy_rp::spi;
use embassy_rp::Peri;
use embassy_rp::bind_interrupts;

use xpanse_driver_api::bus::allocator::SpiResource;
use xpanse_driver_api::bus::spi::{DynSpiBusCombined, SpiBusHandle, SpiBusVersion};

use super::spi_bitbang::BitBangSpiBus;
use super::spi_hardware::HardwareSpiBus;
use super::spi_pio::PioSpiBus;

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO1>;
    PIO2_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO2>;
});

pub struct PioManager {
    pio0: Option<Pio<'static, PIO0>>,
    pio1: Option<Pio<'static, PIO1>>,
    pio2: Option<Pio<'static, PIO2>>,
}

impl PioManager {
    pub fn new(
        pio0: Peri<'static, PIO0>,
        pio1: Peri<'static, PIO1>,
        pio2: Peri<'static, PIO2>,
    ) -> Self {
        Self {
            pio0: Some(Pio::new(pio0, PioIrqs)),
            pio1: Some(Pio::new(pio1, PioIrqs)),
            pio2: Some(Pio::new(pio2, PioIrqs)),
        }
    }

    pub fn has_pio_block(&self, block: u8) -> bool {
        match block {
            0 => self.pio0.is_some(),
            1 => self.pio1.is_some(),
            2 => self.pio2.is_some(),
            _ => false,
        }
    }

    pub fn create_pio_spi<CLK: PioPin, MOSI: PioPin, MISO: PioPin>(
        &mut self,
        block: u8,
        sm: u8,
        clk: Peri<'static, CLK>,
        mosi: Peri<'static, MOSI>,
        miso: Peri<'static, MISO>,
        config: spi::Config,
    ) -> Box<dyn DynSpiBusCombined> {
        match (block, sm) {
            (0, 0) => {
                let pio = self.pio0.take().expect("PIO0 block not available");
                let Pio { mut common, sm0, .. } = pio;
                let bus = PioSpiBus::new(&mut common, sm0, clk, mosi, miso, config);
                Box::new(bus)
            }
            _ => {
                panic!("Unsupported PIO block/sm combination: {}/{}", block, sm);
            }
        }
    }
}

pub fn create_spi_bus(
    resource: SpiResource,
    clk: Peri<'static, impl embassy_rp::spi::ClkPin<embassy_rp::peripherals::SPI1> + PioPin>,
    mosi: Peri<'static, impl embassy_rp::spi::MosiPin<embassy_rp::peripherals::SPI1> + PioPin>,
    miso: Peri<'static, impl embassy_rp::spi::MisoPin<embassy_rp::peripherals::SPI1> + PioPin>,
    config: spi::Config,
    pio_manager: &mut PioManager,
) -> SpiBusHandle {
    match resource {
        SpiResource::HwSpi1(peri) => {
            let bus = HardwareSpiBus::new(peri, clk, mosi, miso, config);
            SpiBusHandle::new(Box::new(bus), SpiBusVersion::Hardware)
        }
        SpiResource::Pio { block, sm } => {
            if pio_manager.has_pio_block(block) {
                let bus = pio_manager.create_pio_spi(block, sm, clk, mosi, miso, config);
                SpiBusHandle::new(bus, SpiBusVersion::Pio)
            } else {
                let bus = BitBangSpiBus::new(
                    clk.into(),
                    mosi.into(),
                    miso.into(),
                    config.frequency,
                );
                SpiBusHandle::new(Box::new(bus), SpiBusVersion::BitBang)
            }
        }
        SpiResource::BitBang => {
            let bus = BitBangSpiBus::new(clk.into(), mosi.into(), miso.into(), config.frequency);
            SpiBusHandle::new(Box::new(bus), SpiBusVersion::BitBang)
        }
    }
}
