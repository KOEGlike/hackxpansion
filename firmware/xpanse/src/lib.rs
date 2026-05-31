#![no_std]

use defmt::info;
use embassy_rp::{
    gpio::AnyPin,
    spi::{self, Blocking, Spi},
};

pub fn init_display(
    rst: AnyPin,
    display_cs: AnyPin,
    dcx: AnyPin,
    miso: AnyPin,
    mosi: AnyPin,
    clk: AnyPin,
) -> _ {
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;
    let spi: Spi<'_, _, Blocking> =
        Spi::new_blocking(p.SPI1, clk, mosi, miso, display_config.clone());
}
