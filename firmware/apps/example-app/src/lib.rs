#![no_std]

extern crate alloc;

use core::pin::Pin;

use alloc::boxed::Box;
use core::future::Future;

use embassy_time::Timer;
use xpanse_api::{
    app::App,
    interfaces::buttons::{A, Button},
    registry::{RegisteredResource, Registry},
};

slint::slint! {
    export component ButtonLoggerUI inherits Window {
        in-out property <int> count: 0;
        Text {
            text: "Clicks: " + root.count;
            color: green;
        }
    }
}

pub struct ButtonLoggerApp {
    button: RegisteredResource<Box<dyn Button<A>>>,
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
            let ui = ButtonLoggerUI::new().unwrap();
            ui.show().unwrap();

            let mut count = 0u32;
            loop {
                self.button.resource.wait_for_pressed().await;
                count += 1;
                ui.set_count(count as i32);
                defmt::info!("button A pressed (count: {})", count);
                Timer::after_millis(50).await;
            }
        })
    }

    fn release(self, registry: &mut Registry) {
        registry.return_resource(self.button);
    }
}
