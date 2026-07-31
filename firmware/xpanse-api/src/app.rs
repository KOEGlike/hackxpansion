//! Application lifecycle and resource ownership.
//!
//! Apps lease capabilities from a [`Registry`] when they are created and must
//! return every lease when they are released. This makes resource availability
//! predictable as the platform switches between apps.

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use crate::registry::Registry;

/// An application that can be selected and run by the xpanse firmware.
///
/// `can_run` should mirror the resource set acquired by `new`, without mutating
/// the registry. After `run` completes, the platform passes the app to
/// `release`, which must return all of its resource leases.
///
/// Apps run on core 1, so any task spawned will also run core 1
///
/// Apps can use slint ui or direct video to show stuff on the screen
///
/// # Example
///
/// ```ignore
/// use std::{boxed::Box, future::Future, pin::Pin};
/// use xpanse_api::{
///     app::App,
///     registry::{Registry, ResourceLease},
/// };
///
/// struct StatusLed;
///
/// struct StatusApp {
///     led: ResourceLease<StatusLed>,
/// }
///
/// impl App for StatusApp {
///     const NAME: &'static str = "Status";
///
///     fn can_run(registry: &Registry) -> bool {
///         registry.has::<StatusLed>()
///     }
///
///     fn new(registry: &mut Registry) -> Option<Self> {
///         Some(Self { led: registry.take_resource()? })
///     }
///
///     fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
///         Box::pin(async move {
///             let _led = self.led.resource_mut();
///             // Drive the capability until the app decides to exit.
///         })
///     }
///
///     fn release(self, registry: &mut Registry) {
///         registry.return_resource(self.led);
///     }
/// }
/// ```
pub trait App: Send {
    /// User-facing name shown by the app picker.
    const NAME: &'static str;

    /// Runs the app until it exits.
    ///
    /// Resource leases held by `self` remain exclusive while this future is
    /// alive. The returned future is not required to be [`Send`].
    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>>;

    /// Reports whether all resources required by this app are available.
    ///
    /// This method must not mutate `registry`. Use
    /// [`Registry::has_resource_set`] when the app needs several distinct
    /// physical resources.
    fn can_run(registry: &Registry) -> bool
    where
        Self: Sized;

    /// Leases the resources required to construct the app.
    ///
    /// Returns `None` if the resources are no longer available. Acquisition of
    /// multiple resources should use [`Registry::take_resource_set`] so it is
    /// atomic.
    fn new(registry: &mut Registry) -> Option<Self>
    where
        Self: Sized;

    /// Returns every resource lease held by the app to `registry`.
    ///
    /// Dropping a lease instead would permanently remove its complete physical
    /// resource group from the registry.
    fn release(self, registry: &mut Registry)
    where
        Self: Sized;
}
