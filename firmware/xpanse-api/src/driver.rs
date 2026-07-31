//! Module driver traits.
//!
//! A driver identifies its hardware through [`DriverMeta::ID`], consumes the
//! detected module's [`crate::gpio_bank::GpioBank`], initializes any buses it
//! needs, and publishes app-facing capabilities in a [`crate::registry::Registry`].

use crate::bus::allocator::BusAllocator;
use crate::gpio_bank::{BankPins, GpioBank};
use crate::metadata::{ModuleID, ModuleSlot};
use crate::registry::Registry;
use core::future::Future;

/// Error returned when a module driver cannot be initialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum DriverError {
    /// Peripheral setup, device communication, or resource registration failed.
    InitFailed,
}

/// Static metadata used to match a driver to a detected module.
pub trait DriverMeta {
    /// Resistor-coded module identifier handled by this driver.
    const ID: ModuleID;
}

/// Initializes a module and publishes the capabilities it provides.
///
/// The platform transfers ownership of the complete [`GpioBank`] to the driver.
/// Bus resources allocated through [`BusAllocator`] are startup allocations and
/// remain owned by the resulting capability for the rest of the boot.
///
/// Drivers are run on core 0, so any task spawn will also run on core 0
///
/// # Example
///
/// A two-button module can consume GPIO 0 and GPIO 1 and publish each button as
/// an independent resource:
///
/// ```ignore
/// use xpanse_api::{
///     bus::allocator::BusAllocator,
///     driver::{Driver, DriverError, DriverMeta},
///     gpio_bank::{BankPins, GpioBank},
///     interfaces::buttons::{A, B, pin_button},
///     metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
///     registry::Registry,
/// };
///
/// struct TwoButtonDriver;
///
/// impl DriverMeta for TwoButtonDriver {
///     const ID: ModuleID = ModuleID {
///         md0: ModuleDetectResistor::R1K6,
///         md1: ModuleDetectResistor::R1K5,
///     };
/// }
///
/// impl<G: BankPins> Driver<G> for TwoButtonDriver {
///     async fn create(
///         bank: GpioBank<G>,
///         slot: ModuleSlot,
///         registry: &mut Registry,
///         buses: &mut BusAllocator,
///     ) -> Result<(), DriverError> {
///         registry.register(slot, Self::ID, pin_button::<A>(bank.gpio0.into()));
///         registry.register(slot, Self::ID, pin_button::<B>(bank.gpio1.into()));
///         let _ = buses;
///         Ok(())
///     }
/// }
/// ```
pub trait Driver<G: BankPins>: DriverMeta {
    /// Consumes a module's pins, initializes its hardware, and registers its
    /// capabilities.
    ///
    /// Implementations should avoid publishing partially initialized resources
    /// and return [`DriverError::InitFailed`] if setup cannot complete.
    fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> impl Future<Output = Result<(), DriverError>>;
}
