#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Level, Output},
    i2c,
};
use static_cell::StaticCell;
use xpanse::{adc::init_adc, display::init_display};
use xpanse_driver_api::gpio_bank::GpioBank;
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Hackxpansion"),
    embassy_rp::binary_info::rp_program_description!(c"Firmware for Hackxpansion"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

embassy_rp::bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

static BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());

    let display_buffer = BUFFER.init([0_u8; 512]);
    let disp = init_display(
        p.SPI0,
        p.PIN_34,
        p.PIN_35,
        p.PIN_5.into(),
        p.PIN_38.into(),
        p.PIN_25.into(),
        display_buffer,
    );

    let config = embassy_rp::i2c::Config::default();
    let bus = embassy_rp::i2c::I2c::new_async(
        p.I2C1.reborrow(),
        p.PIN_15.reborrow(),
        p.PIN_14.reborrow(),
        Irqs,
        config,
    );

    let adc = init_adc(bus).await.unwrap();

    drop(adc);

    let pin = Output::new(p.PIN_15, Level::Low);

    let gpio_bank_0 = GpioBank::new(
        p.PIN_37, //0
        p.PIN_36, //1
        p.PIN_30, //2
        p.PIN_24, //3
        p.PIN_27, //4
        p.PIN_8,  //5
        p.PIN_9,  //6
        p.PIN_46, //7
        p.PIN_47, //8
        p.PIN_39, //9
        p.PWM_SLICE11,
        p.PWM_SLICE4,
        p.PWM_SLICE7,
    );

    let gpio_bank_1 = GpioBank::new(
        p.PIN_19, //0
        p.PIN_18, //1
        p.PIN_22, //2
        p.PIN_32, //3
        p.PIN_23, //4
        p.PIN_28, //5
        p.PIN_29, //6
        p.PIN_44, //7
        p.PIN_45, //8
        p.PIN_26, //9
        p.PWM_SLICE10,
        p.PWM_SLICE6,
        p.PWM_SLICE3,
    );

    let gpio_bank_2 = GpioBank::new(
        p.PIN_7,  //0
        p.PIN_6,  //1
        p.PIN_2,  //2
        p.PIN_4,  //3
        p.PIN_3,  //4
        p.PIN_0,  //5
        p.PIN_1,  //6
        p.PIN_40, //7
        p.PIN_41, //8
        p.PIN_33, //9
        p.PWM_SLICE8,
        p.PWM_SLICE0,
        p.PWM_SLICE1,
    );

    let gpio_bank_3 = GpioBank::new(
        p.PIN_13, //0
        p.PIN_16, //1
        p.PIN_10, //2
        p.PIN_12, //3
        p.PIN_11, //4
        p.PIN_20, //5
        p.PIN_21, //6
        p.PIN_42, //7
        p.PIN_43, //8
        p.PIN_17, //9
        p.PWM_SLICE9,
        p.PWM_SLICE2,
        p.PWM_SLICE5,
    );
}
