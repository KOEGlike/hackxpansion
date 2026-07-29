extern crate alloc;

use alloc::{boxed::Box, rc::Rc};
use core::future::Future;

use embassy_futures::select::{Either, select as select_future};
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor, raw::RawU16},
    primitives::Rectangle,
};

use crate::{
    app_loader::run_app,
    app_picker::{create_app_picker, pick_app},
    core_driver::take_registry,
    display::{self, init_display},
    resource_split::*,
};
use slint::platform::{
    Platform,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel},
};
use static_cell::{ConstStaticCell, StaticCell};
use xpanse_api::interfaces::video;

pub use crate::app_picker::{down, left, right, select, up};

static DRIVER_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static SLINT_BUFFER: ConstStaticCell<
    [Rgb565Pixel; display::WIDTH as usize * display::HIGHT as usize],
> = ConstStaticCell::new([Rgb565Pixel(0); display::WIDTH as usize * display::HIGHT as usize]);

#[embassy_executor::task]
pub async fn ui_core_task(display_peris: DisplayPeris) {
    defmt::info!("ui_core: task started on core 0");
    defmt::info!("ui_core: initializing display");

    let mut backlight = Output::new(display_peris.backlight, Level::Low);
    let driver_buffer = DRIVER_BUFFER.init([0_u8; 512]);
    let mut disp = match init_display(
        display_peris.spi,
        display_peris.clk,
        display_peris.mosi,
        display_peris.rst.into(),
        display_peris.cs.into(),
        display_peris.dc.into(),
        driver_buffer,
    ) {
        Ok(disp) => disp,
        Err(e) => {
            defmt::error!("ui_core: failed to initialize display: {:#?}", e);
            return;
        }
    };
    backlight.set_high();
    defmt::info!("ui_core: display initialized, backlight enabled");

    defmt::info!("ui_core: initializing Slint platform");
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    if slint::platform::set_platform(Box::new(XpansePlatform {
        window: window.clone(),
    }))
    .is_err()
    {
        defmt::error!("ui_core: failed to initialize Slint platform");
        return;
    }

    window.set_size(slint::PhysicalSize::new(
        display::WIDTH as u32,
        display::HIGHT as u32,
    ));

    let slint_buffer = SLINT_BUFFER.take();
    let (frame_buffer, frame_display) = video::indexed_frame_buffer();
    defmt::info!(
        "ui_core: Slint platform ready at {}x{}",
        display::WIDTH,
        display::HIGHT
    );

    defmt::info!("ui_core: waiting for registry from core 1");
    let mut registry = take_registry().await;
    registry.register_platform(frame_buffer);
    defmt::info!("ui_core: registry received");

    let app_picker = match create_app_picker() {
        Ok(app_picker) => app_picker,
        Err(_) => {
            defmt::error!("ui_core: failed to create app picker");
            return;
        }
    };
    defmt::info!("ui_core: boot complete, app picker ready");

    loop {
        let app = drive_ui_until(
            &window,
            &mut disp,
            &mut slint_buffer[..],
            &frame_display,
            pick_app(&mut registry, &app_picker),
        )
        .await;

        defmt::info!("starting selected app: {}", app.name);
        drive_ui_until(
            &window,
            &mut disp,
            &mut slint_buffer[..],
            &frame_display,
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
    frame_display: &video::IndexedFrameDisplay,
    future: F,
) -> F::Output
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
    F: Future,
{
    let ui_future = async {
        let mut direct_frame_token = 0;
        let mut direct_frame_active = false;
        loop {
            render_ui(
                window,
                disp,
                slint_buffer,
                frame_display,
                &mut direct_frame_token,
                &mut direct_frame_active,
            );
            wait_for_next_ui_tick(window).await;
        }
    };

    match select_future(future, ui_future).await {
        Either::First(output) => output,
        Either::Second(_) => unreachable!(),
    }
}

fn render_ui<D>(
    window: &MinimalSoftwareWindow,
    disp: &mut D,
    slint_buffer: &mut [Rgb565Pixel],
    frame_display: &video::IndexedFrameDisplay,
    direct_frame_token: &mut u64,
    direct_frame_active: &mut bool,
) where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    if let Some(frame) = frame_display.active_frame() {
        let token = frame.token();
        if token != *direct_frame_token {
            render_direct_frame(disp, &frame, !*direct_frame_active);
            *direct_frame_token = token;
        }
        *direct_frame_active = true;
        return;
    }

    *direct_frame_active = false;
    *direct_frame_token = 0;
    slint::platform::update_timers_and_animations();

    window.draw_if_needed(|renderer| {
        let dirty_region = renderer.render(slint_buffer, display::WIDTH as usize);
        let mut draw_failed = false;

        for (origin, size) in dirty_region.iter() {
            let x = origin.x as usize;
            let y = origin.y as usize;
            let width = size.width as usize;
            let height = size.height as usize;
            let pixels = (y..y + height).flat_map(|row| {
                let start = row * display::WIDTH as usize + x;
                slint_buffer[start..start + width]
                    .iter()
                    .map(slint_rgb565_into_embedded_graphics)
            });

            if disp
                .fill_contiguous(
                    &Rectangle {
                        top_left: Point::new(origin.x, origin.y),
                        size: Size::new(size.width, size.height),
                    },
                    pixels,
                )
                .is_err()
            {
                draw_failed = true;
                break;
            }
        }

        if draw_failed {
            defmt::error!("ui_core: failed to draw display frame");
            window.request_redraw();
        }
    });
}

fn render_direct_frame<D>(disp: &mut D, frame: &video::PresentedFrame, clear_background: bool)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let width = u32::from(frame.width());
    let height = u32::from(frame.height());
    if width > u32::from(display::WIDTH) || height > u32::from(display::HIGHT) {
        defmt::error!("ui_core: direct framebuffer is larger than the display");
        return;
    }

    if clear_background
        && disp
            .fill_solid(
                &Rectangle::new(
                    Point::zero(),
                    Size::new(u32::from(display::WIDTH), u32::from(display::HIGHT)),
                ),
                Rgb565::BLACK,
            )
            .is_err()
    {
        defmt::error!("ui_core: failed to clear direct framebuffer background");
        return;
    }

    let origin = Point::new(
        ((u32::from(display::WIDTH) - width) / 2) as i32,
        ((u32::from(display::HIGHT) - height) / 2) as i32,
    );
    let pixels = (0..frame.len()).map(|index| {
        let color = frame.color(frame.pixel(index));
        Rgb565::from(RawU16::new(color))
    });
    if disp
        .fill_contiguous(&Rectangle::new(origin, Size::new(width, height)), pixels)
        .is_err()
    {
        defmt::error!("ui_core: failed to draw direct framebuffer");
    }
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
    Rgb565::from(RawU16::new(rgb.0))
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
