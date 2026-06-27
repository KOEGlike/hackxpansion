extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
use embassy_futures::select::select;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::Rgb565,
    primitives::Rectangle,
};

use crate::{
    app_core::take_registry,
    display::{self, init_display},
    resource_split::*,
};
use slint::platform::{
    Platform,
    software_renderer::{MinimalSoftwareWindow, Rgb565Pixel},
};
use static_cell::StaticCell;
use xpanse_driver_api::{app::App, registry::Registry};

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

    spawn_app::<example_app::ButtonLoggerApp>(&mut registry).await;

    loop {
        slint::platform::update_timers_and_animations();

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

        if !window.has_active_animations() {
            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                Timer::after(Duration::from_nanos(duration.as_nanos() as u64)).await;
            }
        }
    }
}

async fn spawn_app<A: App + 'static>(registry: &mut Registry) {
    if A::can_run(registry) {
        if let Some(app) = A::new(registry) {
            run_app(app).await;
            defmt::info!("app started");
        }
    } else {
        defmt::warn!("app requirements not met");
    }
}

async fn run_app(mut app: impl App) {
    let app_future = app.run();
    let ui_future = async {
        loop {
            slint::platform::update_timers_and_animations();
            slint::platform::duration_until_next_timer_update();
            Timer::after_millis(10).await;
        }
    };
    select(app_future, ui_future).await;
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
