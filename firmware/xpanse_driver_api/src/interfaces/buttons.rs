pub trait Button {
    fn wait_for_pressed(&mut self) -> impl Future<Output = ()>;
}

pub trait ButtonUp: Button {}
pub trait ButtonDown: Button {}
pub trait ButtonLeft: Button {}
pub trait ButtonRight: Button {}

pub trait ButtonA: Button {}
pub trait ButtonB: Button {}
pub trait ButtonX: Button {}
pub trait ButtonY: Button {}
