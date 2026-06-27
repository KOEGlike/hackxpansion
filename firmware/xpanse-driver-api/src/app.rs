use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;

use crate::registry::Registry;

pub trait App: Send {
    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>>;

    fn can_run(registry: &Registry) -> bool
    where
        Self: Sized;

    fn new(registry: &mut Registry) -> Option<Self>
    where
        Self: Sized;
}
