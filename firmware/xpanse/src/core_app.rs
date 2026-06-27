extern crate alloc;

use alloc::{boxed::Box, rc::Rc};
use core::future::Future;

use embassy_futures::select::{Either, select as select_future};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::Rgb565,
    primitives::Rectangle,
};

use crate::{
    app_loader::run_app,
    app_picker::pick_app,
    core_driver::take_registry,
    display::{self, init_display},
    resource_split::*,
};
use slint::platform::{
    Platform,
    software_renderer::{MinimalSoftwareWindow, Rgb565Pixel},
};
use static_cell::StaticCell;

pub use crate::app_picker::{down, left, right, select, up};

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
    slint::platform::set_platform(Box::new(XpansePlatform {
        window: window.clone(),
    }))
    .unwrap();

    window.set_size(slint::PhysicalSize::new(
        display::WIDTH as u32,
        display::HIGHT as u32,
    ));

    let slint_buffer =
        SLINT_BUFFER.init([Rgb565Pixel(0); display::WIDTH as usize * display::HIGHT as usize]);

    defmt::info!("ui_core: waiting for registry from core 1");
    let mut registry = take_registry().await;
    defmt::info!("ui_core: registry received");

    loop {
        let app = drive_ui_until(
            &window,
            &mut disp,
            &mut slint_buffer[..],
            pick_app(&registry),
        )
        .await;

        defmt::info!("starting selected app: {}", app.name);
        drive_ui_until(
            &window,
            &mut disp,
            &mut slint_buffer[..],
            run_app(app, &mut registry),
        )
        .await;
        defmt::info!("selected app returned");
    }
}

async fn drive_ui_until<D, F>(
    window: &MinimalSoftwareWindow,
    disp: &mut D,
    slint_buffer: &mut [Rgb565Pixel],
    future: F,
) -> F::Output
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
    F: Future,
{
    let ui_future = async {
        loop {
            render_ui(window, disp, slint_buffer);
            wait_for_next_ui_tick(window).await;
        }
    };

    match select_future(future, ui_future).await {
        Either::First(output) => output,
        Either::Second(_) => unreachable!(),
    }
}

fn render_ui<D>(window: &MinimalSoftwareWindow, disp: &mut D, slint_buffer: &mut [Rgb565Pixel])
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    slint::platform::update_timers_and_animations();

    window.draw_if_needed(|renderer| {
        renderer.render(slint_buffer, display::WIDTH as usize);
        disp.fill_contiguous(
            &Rectangle {
                top_left: Point::zero(),
                size: Size::new(display::WIDTH as u32, display::HIGHT as u32),
            },
            slint_buffer.iter().map(slint_rgb565_into_embedded_graphics),
        )
        .unwrap();
    });
}

async fn wait_for_next_ui_tick(window: &MinimalSoftwareWindow) {
    if window.has_active_animations() {
        Timer::after_millis(10).await;
        return;
    }

    if let Some(duration) = slint::platform::duration_until_next_timer_update() {
        Timer::after(Duration::from_nanos(duration.as_nanos() as u64)).await;
    } else {
        Timer::after_millis(10).await;
    }
}

fn slint_rgb565_into_embedded_graphics(rgb: &Rgb565Pixel) -> Rgb565 {
    let rgb: rgb::RGB<u8> = (*rgb).into();
    Rgb565::new(rgb.r, rgb.g, rgb.b)
}

struct XpansePlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for XpansePlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        Instant::now().duration_since(Instant::from_secs(0)).into()
    }
}
