use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;

pub trait Button {
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

pub trait ButtonUp: Button {}
pub trait ButtonDown: Button {}
pub trait ButtonLeft: Button {}
pub trait ButtonRight: Button {}

pub trait ButtonA: Button {}
pub trait ButtonB: Button {}
pub trait ButtonX: Button {}
pub trait ButtonY: Button {}
