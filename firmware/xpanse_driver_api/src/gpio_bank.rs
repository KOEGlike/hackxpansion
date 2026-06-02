use embassy_rp::{Peri, peripherals::{PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9}, spi::{Instance, MisoPin}};

pub struct GpioBank<'a, I:Instance> {
    gpio0: Peri<'a, impl MisoPin<I>>,
    gpio1: Peri<'a, PIN_1>,
    gpio2: Peri<'a, PIN_2>,
    gpio3: Peri<'a, PIN_3>,
    gpio4: Peri<'a, PIN_4>,
    gpio5: Peri<'a, PIN_5>,
    gpio6
    gpio7
    gpio8
    gpio9
}