use static_cell::StaticCell;

use crate::{display::init_display, resource_split::*};

static BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

#[embassy_executor::task]
pub async fn core0_task(display_peris: DisplayPeris) {
    let display_buffer = BUFFER.init([0_u8; 512]);
    let disp = init_display(
        display_peris.spi,
        display_peris.clk,
        display_peris.mosi,
        display_peris.rst.into(),
        display_peris.cs.into(),
        display_peris.dc.into(),
        display_buffer,
    );
}
