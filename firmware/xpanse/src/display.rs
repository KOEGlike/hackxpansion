use embassy_rp::{
    Peri,
    gpio::{AnyPin, Level, Output},
    spi::{self, ClkPin, Instance, MosiPin, Spi},
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::interface::SpiInterface;
use mipidsi::{
    Builder,
    models::ST7789,
    options::{Orientation, Rotation},
};

pub const HIGHT: u16 = 240;
pub const WIDTH: u16 = 320;

// Update your type definition to use ExclusiveDevice
pub type Display<T> = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, T, spi::Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    Output<'static>,
>;

pub fn init_display<T: Instance>(
    spi: Peri<'static, T>,
    clk: Peri<'static, impl ClkPin<T>>,
    mosi: Peri<'static, impl MosiPin<T>>,
    rst: Peri<'static, AnyPin>,
    display_cs: Peri<'static, AnyPin>,
    dcx: Peri<'static, AnyPin>,
    buffer: &'static mut [u8],
) -> Display<T> {
    let mut display_config = spi::Config::default();
    display_config.frequency = 64_000_000;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    // Construct the SPI driver
    let spi = Spi::new_blocking_txonly(spi, clk, mosi, display_config);

    let dcx = Output::new(dcx, Level::Low);
    let rst = Output::new(rst, Level::Low);
    let display_cs = Output::new(display_cs, Level::High);

    // Give the display exclusive ownership of the SPI bus
    let display_spi = ExclusiveDevice::new(spi, display_cs, Delay).unwrap();

    // Display interface abstraction from SPI and DC
    let di = SpiInterface::new(display_spi, dcx, buffer);

    Builder::new(ST7789, di)
        .display_size(WIDTH, HIGHT)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut Delay)
        .unwrap()
}
