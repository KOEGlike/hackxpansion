extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use xpanse_driver_api::{app::App, registry::Registry};

const APP_CATALOG: &[AppDescriptor] = &[AppDescriptor {
    name: example_app::ButtonLoggerApp::NAME,
    can_run: example_app::ButtonLoggerApp::can_run,
    run: run_app_impl::<example_app::ButtonLoggerApp>,
}];

type AppFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;
type AppRunner = for<'a> fn(&'a mut Registry) -> AppFuture<'a>;

pub(crate) struct AppDescriptor {
    pub name: &'static str,
    can_run: fn(&Registry) -> bool,
    run: AppRunner,
}

pub(crate) fn runnable_apps<'a>(
    registry: &'a Registry,
) -> impl Iterator<Item = &'static AppDescriptor> + 'a {
    APP_CATALOG
        .iter()
        .filter(move |app| (app.can_run)(registry))
}

pub(crate) fn runnable_app_count(registry: &Registry) -> usize {
    runnable_apps(registry).count()
}

pub(crate) fn runnable_app_at(
    registry: &Registry,
    selected_index: usize,
) -> Option<&'static AppDescriptor> {
    runnable_apps(registry).nth(selected_index)
}

pub(crate) fn run_app<'a>(
    app: &'static AppDescriptor,
    registry: &'a mut Registry,
) -> AppFuture<'a> {
    (app.run)(registry)
}

fn run_app_impl<'a, A: App + 'static>(registry: &'a mut Registry) -> AppFuture<'a> {
    Box::pin(async move {
        if let Some(mut app) = A::new(registry) {
            app.run().await;
            app.release(registry);
        } else {
            defmt::warn!("selected app requirements were no longer met");
        }
    })
}
