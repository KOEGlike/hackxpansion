#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use embassy_futures::select::{Either, Either6, select, select6};
use embassy_time::Timer;
use slint::{Color, ComponentHandle};
use xpanse_api::{
    app::App,
    interfaces::buttons::{A, B, Button, Down, Left, Right, Up},
    registry::{Registry, ResourceLease},
};

slint::include_modules!();

const STEP: i32 = 8;
const REPEAT_INTERVAL_MS: u64 = 50;
const MAX_X: i32 = 280;
const MAX_Y: i32 = 200;
const COLORS: [Color; 6] = [
    Color::from_rgb_u8(229, 57, 53),
    Color::from_rgb_u8(30, 136, 229),
    Color::from_rgb_u8(67, 160, 71),
    Color::from_rgb_u8(251, 140, 0),
    Color::from_rgb_u8(142, 36, 170),
    Color::from_rgb_u8(0, 137, 123),
];

type AppButton<R> = ResourceLease<Box<dyn Button<R>>>;
type CubeControls = (
    Box<dyn Button<Up>>,
    Box<dyn Button<Down>>,
    Box<dyn Button<Left>>,
    Box<dyn Button<Right>>,
    Box<dyn Button<A>>,
    Box<dyn Button<B>>,
);

pub struct CubeGameApp {
    up: AppButton<Up>,
    down: AppButton<Down>,
    left: AppButton<Left>,
    right: AppButton<Right>,
    color: AppButton<A>,
    exit: AppButton<B>,
}

impl App for CubeGameApp {
    const NAME: &'static str = "Cube Game";

    fn can_run(registry: &Registry) -> bool {
        registry.has_resource_set::<CubeControls>()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let (up, down, left, right, color, exit) = registry.take_resource_set::<CubeControls>()?;

        Some(Self {
            up,
            down,
            left,
            right,
            color,
            exit,
        })
    }

    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let ui = match CubeGameUI::new() {
                Ok(ui) => ui,
                Err(_) => {
                    defmt::error!("CubeGameApp: failed to create UI");
                    return;
                }
            };
            if ui.show().is_err() {
                defmt::error!("CubeGameApp: failed to show UI");
                return;
            }

            let mut x = 144;
            let mut y = 104;
            let mut color_index = 0;
            ui.set_cube_color(COLORS[color_index]);

            'game: loop {
                let input = select(
                    select6(
                        self.up.resource_mut().wait_for_pressed(),
                        self.down.resource_mut().wait_for_pressed(),
                        self.left.resource_mut().wait_for_pressed(),
                        self.right.resource_mut().wait_for_pressed(),
                        self.color.resource_mut().wait_for_pressed(),
                        self.exit.resource_mut().wait_for_pressed(),
                    ),
                    Timer::after_millis(REPEAT_INTERVAL_MS),
                )
                .await;

                match input {
                    Either::First(Either6::First(())) => y = (y - STEP).max(0),
                    Either::First(Either6::Second(())) => y = (y + STEP).min(MAX_Y),
                    Either::First(Either6::Third(())) => x = (x - STEP).max(0),
                    Either::First(Either6::Fourth(())) => x = (x + STEP).min(MAX_X),
                    Either::First(Either6::Fifth(())) => {
                        color_index = (color_index + 1) % COLORS.len();
                        ui.set_cube_color(COLORS[color_index]);
                    }
                    Either::First(Either6::Sixth(())) => break 'game,
                    Either::Second(()) => {
                        let horizontal = i32::from(self.right.resource().is_pressed())
                            - i32::from(self.left.resource().is_pressed());
                        let vertical = i32::from(self.down.resource().is_pressed())
                            - i32::from(self.up.resource().is_pressed());
                        x = (x + horizontal * STEP).clamp(0, MAX_X);
                        y = (y + vertical * STEP).clamp(0, MAX_Y);
                    }
                }

                ui.set_cube_x(x);
                ui.set_cube_y(y);
            }

            if ui.hide().is_err() {
                defmt::error!("CubeGameApp: failed to hide UI");
            }
        })
    }

    fn release(self, registry: &mut Registry) {
        registry.return_resource(self.up);
        registry.return_resource(self.down);
        registry.return_resource(self.left);
        registry.return_resource(self.right);
        registry.return_resource(self.color);
        registry.return_resource(self.exit);
    }
}
