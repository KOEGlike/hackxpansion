use display_interface_spi::SPIInterface;
use embassy_rp::{
    Peri,
    gpio::{AnyPin, Level, Output},
    spi::{self, ClkPin, Instance, MosiPin, Spi},
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::{
    Builder,
    models::ST7789,
    options::{Orientation, Rotation},
};

// Update your type definition to use ExclusiveDevice
type Display<'a, T: Instance> = mipidsi::Display<
    SPIInterface<ExclusiveDevice<Spi<'a, T, spi::Blocking>, Output<'a>, Delay>, Output<'a>>,
    ST7789,
    Output<'a>,
>;

pub fn init_display<'d, T: Instance>(
    spi: Peri<'d, T>,
    clk: Peri<'d, impl ClkPin<T>>,
    mosi: Peri<'d, impl MosiPin<T>>,
    rst: Peri<'d, AnyPin>,
    display_cs: Peri<'d, AnyPin>,
    dcx: Peri<'d, AnyPin>,
) -> Display<'d, T> {
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
    let display_spi = ExclusiveDevice::new(spi, display_cs, Delay);

    // Display interface abstraction from SPI and DC
    let di = SPIInterface::new(display_spi, dcx);

    Builder::new(ST7789, di)
        .display_size(240, 320)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut Delay)
        .unwrap()
}
