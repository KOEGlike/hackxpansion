#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use embassy_time::Timer;
use slint::ComponentHandle;
use xpanse_api::{
    app::App,
    interfaces::buttons::{A, Button},
    registry::{Registry, ResourceLease},
};

slint::include_modules!();

pub struct ButtonLoggerApp {
    button: ResourceLease<Box<dyn Button<A>>>,
}

impl App for ButtonLoggerApp {
    const NAME: &'static str = "Button Logger";

    fn can_run(registry: &Registry) -> bool {
        registry.has::<Box<dyn Button<A>>>()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let button = registry.take_resource::<Box<dyn Button<A>>>()?;
        Some(Self { button })
    }

    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let ui = match ButtonLoggerUI::new() {
                Ok(ui) => ui,
                Err(_) => {
                    defmt::error!("ButtonLoggerApp: failed to create UI");
                    return;
                }
            };
            if ui.show().is_err() {
                defmt::error!("ButtonLoggerApp: failed to show UI");
                return;
            }

            let mut count = 0u32;
            loop {
                self.button.resource_mut().wait_for_pressed().await;

                let mut held_ms = 0;
                while self.button.resource().is_pressed() && held_ms < 1_000 {
                    Timer::after_millis(20).await;
                    held_ms += 20;
                }

                if held_ms >= 1_000 {
                    while self.button.resource().is_pressed() {
                        Timer::after_millis(20).await;
                    }
                    break;
                }

                count += 1;
                ui.set_count(count as i32);
                defmt::info!("button A pressed (count: {})", count);
            }

            if ui.hide().is_err() {
                defmt::error!("ButtonLoggerApp: failed to hide UI");
            }
        })
    }

    fn release(self, registry: &mut Registry) {
        registry.return_resource(self.button);
    }
}
