use alloc::boxed::Box;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;

use embassy_rp::{
    Peri,
    gpio::{AnyPin, Input, Pull},
};

mod private {
    pub trait Sealed {}
}

pub trait ButtonRole: private::Sealed + 'static + Send {}

macro_rules! role {
    ($($n:ident),* $(,)?) => {
        $(
            pub struct $n;
            impl private::Sealed for $n {}
            impl ButtonRole for $n {}
        )*
    };
}

role!(A, B, X, Y, Up, Down, Left, Right);

pub trait Button<R: ButtonRole>: Send {
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

pub struct SingleButton<R: ButtonRole> {
    pin: Input<'static>,
    _role: PhantomData<R>,
}

impl<R: ButtonRole> SingleButton<R> {
    pub fn new(pin: Peri<'static, AnyPin>) -> Self {
        Self {
            pin: Input::new(pin, Pull::Up),
            _role: PhantomData,
        }
    }
}

impl<R: ButtonRole> Button<R> for SingleButton<R> {
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.pin.wait_for_low().await;
        })
    }
}

pub fn pin_button<R: ButtonRole>(pin: Peri<'static, AnyPin>) -> Box<dyn Button<R>> {
    Box::new(SingleButton::<R>::new(pin))
}
