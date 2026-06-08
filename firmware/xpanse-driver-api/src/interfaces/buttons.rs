use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

pub struct ButtonUseCase {
    pub button_a: bool,
    pub button_b: bool,
    pub button_x: bool,
    pub button_y: bool,
    pub button_up: bool,
    pub button_down: bool,
    pub button_left: bool,
    pub button_right: bool,
}

pub trait Button {
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
    fn use_case(&self) -> ButtonUseCase;
}
