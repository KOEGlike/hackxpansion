use alloc::{boxed::Box, sync::Arc};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;

use embassy_rp::{
    Peri,
    gpio::{AnyPin, Input, Pull},
};
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::Timer;

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
    fn is_pressed(&self) -> bool;
}

pub struct SingleButton<R: ButtonRole> {
    pin: Input<'static>,
    _role: PhantomData<R>,
}

struct SharedButton<R: ButtonRole> {
    pin: Arc<CriticalSectionMutex<Input<'static>>>,
    _role: PhantomData<R>,
}

impl<R: ButtonRole> SharedButton<R> {
    fn is_low(&self) -> bool {
        self.pin.lock(Input::is_low)
    }
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
            while self.pin.is_low() {
                self.pin.wait_for_high().await;
                Timer::after_millis(20).await;
            }

            loop {
                self.pin.wait_for_low().await;
                Timer::after_millis(20).await;
                if self.pin.is_low() {
                    return;
                }
            }
        })
    }

    fn is_pressed(&self) -> bool {
        self.pin.is_low()
    }
}

impl<R: ButtonRole> Button<R> for SharedButton<R> {
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            while self.is_low() {
                Timer::after_millis(20).await;
            }

            loop {
                Timer::after_millis(20).await;
                if self.is_low() {
                    return;
                }
            }
        })
    }

    fn is_pressed(&self) -> bool {
        self.is_low()
    }
}

pub fn pin_button<R: ButtonRole>(pin: Peri<'static, AnyPin>) -> Box<dyn Button<R>> {
    Box::new(SingleButton::<R>::new(pin))
}

/// Creates two logical button roles backed by one physical GPIO input.
pub fn aliased_pin_buttons<R: ButtonRole, Alias: ButtonRole>(
    pin: Peri<'static, AnyPin>,
) -> (Box<dyn Button<R>>, Box<dyn Button<Alias>>) {
    let pin = Arc::new(CriticalSectionMutex::new(Input::new(pin, Pull::Up)));
    (
        Box::new(SharedButton::<R> {
            pin: pin.clone(),
            _role: PhantomData,
        }),
        Box::new(SharedButton::<Alias> {
            pin,
            _role: PhantomData,
        }),
    )
}
