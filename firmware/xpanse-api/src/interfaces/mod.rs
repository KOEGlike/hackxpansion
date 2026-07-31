//! App- and driver-facing hardware capabilities.
//!
//! These interfaces hide board-specific implementations behind stable resource
//! types that can be registered by drivers and leased by apps.

/// Shared RP235x ADC service.
pub mod adc;
/// Active-low button capabilities and role markers.
pub mod buttons;
/// USB peripheral resources.
pub mod usb;
/// Direct RGB565 framebuffer resources.
pub mod video;
