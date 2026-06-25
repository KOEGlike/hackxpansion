#![no_std]

extern crate alloc;

use core::pin::Pin;

use alloc::boxed::Box;
use core::future::Future;

use embassy_time::Timer;
use xpanse_driver_api::{
    app::App,
    interfaces::buttons::{Button, A},
    registry::Registry,
};

pub struct ButtonLoggerApp {
    button: Box<dyn Button<A>>,
}

impl App for ButtonLoggerApp {
    fn can_run(registry: &Registry) -> bool {
        registry.has::<Box<dyn Button<A>>>()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let button = registry.take::<Box<dyn Button<A>>>()?;
        Some(Self { button })
    }

    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let mut count = 0u32;
            loop {
                self.button.wait_for_pressed().await;
                count += 1;
                defmt::info!("button A pressed (count: {})", count);
                Timer::after_millis(50).await;
            }
        })
    }
}
