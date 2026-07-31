//! GPIO button input wrappers.
//!
//! Provides logical button roles that drivers can publish through the
//! [`crate::registry::Registry`] and that apps can wait on for press events.
//! Each role is a zero-sized marker type; the physical pin is wired to a role
//! at startup using `pin_button` or `aliased_pin_buttons`.

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

/// Marker trait for a logical button role (e.g. `A`, `B`, `Up`).
///
/// The type itself is zero-sized; it exists only to name a resource slot in the
/// [`crate::registry::Registry`].
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

/// Async interface for waiting on a button press and querying its state.
pub trait Button<R: ButtonRole>: Send {
    /// Wait until the button transitions to the pressed state.
    ///
    /// The returned future resolves once a debounced press is detected.
    fn wait_for_pressed<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    /// Returns `true` if the button is currently pressed.
    fn is_pressed(&self) -> bool;
}

/// A single button backed by one GPIO input.
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
    /// Create a `SingleButton` from a GPIO pin configured with a pull-up.
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

/// Create a boxed [`Button`] of role `R` from a single GPIO pin.
///
/// The pin is consumed and configured internally with a pull-up resistor.
///
/// # Example
///
/// ```ignore
/// use embassy_rp::gpio::AnyPin;
/// use embassy_rp::Peri;
/// use xpanse_api::interfaces::buttons::{A, pin_button};
/// use xpanse_api::registry::Registry;
///
/// # async fn example(
/// #     pin: Peri<'static, AnyPin>,
/// #     registry: &mut Registry,
/// #     slot: xpanse_api::metadata::ModuleSlot,
/// # ) {
/// let button = pin_button::<A>(pin);
/// // registry.register(slot, id, button);
/// # }
/// ```
pub fn pin_button<R: ButtonRole>(pin: Peri<'static, AnyPin>) -> Box<dyn Button<R>> {
    Box::new(SingleButton::<R>::new(pin))
}

/// Creates two logical button roles backed by one physical GPIO input.
///
/// This is useful when a single physical button (e.g. a side button) needs to
/// serve two roles simultaneously, such as `A` and `Up`.
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
