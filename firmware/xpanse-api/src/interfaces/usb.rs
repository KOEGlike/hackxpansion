//! USB resources available to apps.

use embassy_rp::{Peri, peripherals::USB};

/// The board's USB peripheral token as stored in the resource registry.
pub type UsbPeripheral = Peri<'static, USB>;

embassy_rp::bind_interrupts!(pub struct UsbIrqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});
