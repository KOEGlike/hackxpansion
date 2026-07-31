//! Bus allocation, type-erased handles, and RP235x bus backends.
//!
//! Drivers normally construct buses through `allocator::BusAllocator`. The
//! allocator chooses or reserves startup resources, while the I2C, SPI, and
//! UART handle modules expose stable interfaces that apps and device drivers can
//! store without naming a concrete backend.

/// Startup allocator for hardware peripherals, DMA channels, and PIO state machines.
pub mod allocator;
/// Type-erased async I2C interface.
pub mod i2c;
/// GPIO bit-banged I2C backend.
pub mod i2c_bitbang;
/// Interrupt-driven RP I2C backend.
pub mod i2c_hardware;
/// PIO state-machine allocation and custom-program access.
pub mod pio;
/// Type-erased blocking and async SPI interface.
pub mod spi;
/// GPIO bit-banged SPI backend.
pub mod spi_bitbang;
/// DMA-backed RP SPI backend.
pub mod spi_hardware;
/// DMA-backed PIO SPI backend.
pub mod spi_pio;
/// Type-erased async UART interface.
pub mod uart;
/// GPIO bit-banged UART backend.
pub mod uart_bitbang;
/// DMA-backed RP UART backend.
pub mod uart_hardware;
/// PIO UART backend.
pub mod uart_pio;
