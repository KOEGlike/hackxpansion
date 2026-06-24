extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::Rgb565,
    primitives::Rectangle,
};

use crate::{
    display::{self, init_display},
    resource_split::*,
};
use slint::platform::{
    Platform,
    software_renderer::{MinimalSoftwareWindow, Rgb565Pixel},
};
use static_cell::StaticCell;

static DRIVER_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static SLINT_BUFFER: StaticCell<[Rgb565Pixel; display::WIDTH as usize * display::HIGHT as usize]> =
    StaticCell::new();

#[embassy_executor::task]
pub async fn ui_core_task(display_peris: DisplayPeris) {
    let driver_buffer = DRIVER_BUFFER.init([0_u8; 512]);
    let mut disp = init_display(
        display_peris.spi,
        display_peris.clk,
        display_peris.mosi,
        display_peris.rst.into(),
        display_peris.cs.into(),
        display_peris.dc.into(),
        driver_buffer,
    );

    let window = MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(XpansePlatfrom {
        window: window.clone(),
    }))
    .unwrap();

    window.set_size(slint::PhysicalSize::new(
        display::WIDTH as u32,
        display::HIGHT as u32,
    ));

    let slint_buffer =
        SLINT_BUFFER.init([Rgb565Pixel(0); display::WIDTH as usize * display::HIGHT as usize]);

    loop {
        // Let Slint run the timer hooks and update animations.
        slint::platform::update_timers_and_animations();

        // ... maybe some more application logic ...

        // Draw the scene if something needs to be drawn.
        window.draw_if_needed(|renderer| {
            renderer.render(&mut slint_buffer[..], display::WIDTH as usize);
            disp.fill_contiguous(
                &Rectangle {
                    top_left: Point::zero(),
                    size: Size::new(display::WIDTH as u32, display::HIGHT as u32),
                },
                slint_buffer.iter().map(slint_rgb565_into_embedded_graphics),
            )
            .unwrap();
        });

        // Try to put the MCU to sleep
        if !window.has_active_animations() {
            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                Timer::after(Duration::from_nanos(duration.as_nanos() as u64)).await;
            }
        }
    }
}

fn slint_rgb565_into_embedded_graphics(rgb: &Rgb565Pixel) -> Rgb565 {
    let rgb: rgb::RGB<u8> = rgb.clone().into();
    Rgb565::new(rgb.r, rgb.g, rgb.b)
}

struct XpansePlatfrom {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for XpansePlatfrom {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        // Since on MCUs, there can be only one window, just return a clone of self.window.
        // We'll also use the same window in the event loop.
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        Instant::now().duration_since(Instant::from_secs(0)).into()
    }
}
