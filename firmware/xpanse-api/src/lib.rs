#![no_std]
extern crate alloc;

/// Exact versions of dependencies whose types and traits appear in this crate's public API.
pub mod reexports {
    pub use embassy_rp;
    pub use embedded_hal;
    pub use embedded_hal_async;
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
