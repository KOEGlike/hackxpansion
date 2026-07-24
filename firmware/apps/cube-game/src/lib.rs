#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{future::Future, pin::Pin};

use embassy_futures::select::{Either, Either6, select, select6};
use embassy_time::Timer;
use slint::{Color, ComponentHandle};
use xpanse_api::{
    app::App,
    interfaces::buttons::{A, B, Button, ButtonRole, Down, Left, Right, Up},
    registry::{RegisteredResource, Registry, ResourceId},
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

type AppButton<R> = RegisteredResource<Box<dyn Button<R>>>;
type OptionalButton<R> = Option<AppButton<R>>;

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
        control_assignment(registry).is_some()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let ids = control_assignment(registry)?;
        let up = take_button_with_id(registry, ids[0]);
        let down = take_button_with_id(registry, ids[1]);
        let left = take_button_with_id(registry, ids[2]);
        let right = take_button_with_id(registry, ids[3]);
        let color = take_button_with_id(registry, ids[4]);
        let exit = take_button_with_id(registry, ids[5]);

        match (up, down, left, right, color, exit) {
            (Some(up), Some(down), Some(left), Some(right), Some(color), Some(exit)) => {
                Some(Self {
                    up,
                    down,
                    left,
                    right,
                    color,
                    exit,
                })
            }
            (up, down, left, right, color, exit) => {
                return_button(registry, up);
                return_button(registry, down);
                return_button(registry, left);
                return_button(registry, right);
                return_button(registry, color);
                return_button(registry, exit);
                None
            }
        }
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
                        self.up.resource.wait_for_pressed(),
                        self.down.resource.wait_for_pressed(),
                        self.left.resource.wait_for_pressed(),
                        self.right.resource.wait_for_pressed(),
                        self.color.resource.wait_for_pressed(),
                        self.exit.resource.wait_for_pressed(),
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
                        let horizontal = i32::from(self.right.resource.is_pressed())
                            - i32::from(self.left.resource.is_pressed());
                        let vertical = i32::from(self.down.resource.is_pressed())
                            - i32::from(self.up.resource.is_pressed());
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

fn resource_ids<R: ButtonRole>(registry: &Registry) -> Vec<ResourceId> {
    registry
        .capabilities::<Box<dyn Button<R>>>()
        .map(|buttons| buttons.iter().map(RegisteredResource::id).collect())
        .unwrap_or_default()
}

fn control_assignment(registry: &Registry) -> Option<Vec<ResourceId>> {
    let candidates = [
        resource_ids::<Up>(registry),
        resource_ids::<Down>(registry),
        resource_ids::<Left>(registry),
        resource_ids::<Right>(registry),
        resource_ids::<A>(registry),
        resource_ids::<B>(registry),
    ];
    let mut assignment = Vec::new();
    can_assign_distinct(&candidates, 0, &mut assignment).then_some(assignment)
}

fn can_assign_distinct(
    candidates: &[Vec<ResourceId>],
    index: usize,
    used: &mut Vec<ResourceId>,
) -> bool {
    if index == candidates.len() {
        return true;
    }

    for id in &candidates[index] {
        if !used.contains(id) {
            used.push(*id);
            if can_assign_distinct(candidates, index + 1, used) {
                return true;
            }
            used.pop();
        }
    }

    false
}

fn take_button_with_id<R: ButtonRole>(
    registry: &mut Registry,
    id: ResourceId,
) -> OptionalButton<R> {
    let mut others = Vec::new();
    let button = loop {
        match registry.take_resource::<Box<dyn Button<R>>>() {
            Some(button) if button.id() == id => break Some(button),
            Some(other) => others.push(other),
            None => break None,
        }
    };

    for other in others {
        registry.return_resource(other);
    }

    button
}

fn return_button<R: ButtonRole>(registry: &mut Registry, button: OptionalButton<R>) {
    if let Some(button) = button {
        registry.return_resource(button);
    }
}
