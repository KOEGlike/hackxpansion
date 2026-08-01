//! Shared API for xpanse apps, module drivers, and platform firmware.
//!
//! Module drivers implement [`driver::Driver`] to turn an owned
//! [`gpio_bank::GpioBank`] into capabilities stored in a [`registry::Registry`].
//! Apps implement [`app::App`] and lease those capabilities for the duration of
//! their run. The platform constructs buses through [`bus::allocator::BusAllocator`]
//! while drivers are being initialized.
//!
//! # Runtime requirements
//!
//! This crate is `no_std`, but uses `alloc` for registry storage, trait objects,
//! and boxed futures. Firmware using the API must install a global allocator
//! before creating drivers, resources, or apps.
//!
//! # Typical lifecycle
//!
//! 1. Detect a module and select a driver using [`metadata::ModuleID`].
//! 2. Call [`driver::Driver::create`] with the module's GPIO bank.
//! 3. Check [`app::App::can_run`] and construct an app with [`app::App::new`].
//! 4. Run the app, then call [`app::App::release`] to return its resource leases.
//!
//! Exact versions of HAL traits used by this API are available through
//! [`reexports`], avoiding version mismatches in app and driver crates.

#![no_std]
extern crate alloc;

/// Exact versions of dependencies whose types and traits appear in this crate's public API.
///
/// For example, import the async I2C trait implemented by
/// `bus::i2c::I2cBusHandle` from this module:
///
/// ```ignore
/// use xpanse_api::reexports::embedded_hal_async::i2c::I2c;
/// ```
pub mod reexports {
    pub use defmt;
    pub use embassy_futures;
    pub use embassy_rp;
    pub use embassy_time;
    pub use embedded_hal;
    pub use embedded_hal_async;
    pub use embedded_hal_bus;
    pub use embedded_io_async;
    pub use slint;
}

pub mod app;
pub mod bus;
pub mod driver;
pub mod gpio_bank;
pub mod interfaces;
pub mod metadata;
pub mod registry;
