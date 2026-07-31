//! Hardware bus allocator.
//!
//! The allocator hands out peripherals (SPI/I2C/UART), PIO state machines and
//! DMA channels to drivers. Constructed bus handles own their resources for the
//! rest of the boot; the allocator is intentionally a startup-time allocator,
//! not a hot-plug pool. The API is designed so that the most common ways to
//! misconfigure hardware are caught at compile time:
//!
//! * Pin roles can't be swapped — `clk`/`mosi`/`miso` and `tx`/`rx` are
//!   distinguished by trait bounds (`ClkPin<I>`, `MosiPin<I>`, …). A pin only
//!   implements the role trait for its actual function, so passing `miso` where
//!   `clk` is expected is a type error.
//! * Pins can't be paired with the wrong peripheral instance — the role traits
//!   are parameterized by the instance (`ClkPin<SPI0>` vs `ClkPin<SPI1>`), so an
//!   SPI0 pin can't be used with the SPI1 peripheral.
//! * The backend can't be confused — there are separate `create_*_hardware`,
//!   `create_*_pio` and `create_*_bitbang` methods. There is no `bool` flag and
//!   no silent HW→bitbang downgrade when a hardware peripheral was specifically
//!   requested.
//! * DMA channels are owned by the allocator — callers specify *which* typed
//!   channel to use, and the allocator tracks availability. A channel already
//!   handed out returns `Err(Exhausted)`.
//! * PIO state machines are allocated soundly — the `PioManager` keeps every SM
//!   alive, so all 12 SMs are independently available and no `(block, sm)`
//!   combination the allocator returns is unbuild-able.
//!
//! Async SPI (hardware and PIO) requires DMA. Because embassy-rp models each
//! DMA channel as a distinct type, the caller specifies the channel types
//! (`TxDma`, `RxDma`) and provides the board's IRQ binding; the allocator
//! dispenses the matching `Peri` tokens from its pool. PIO UART uses FIFO
//! polling (no DMA), so [`BusAllocator::create_uart`](crate::bus::allocator::BusAllocator::create_uart) provides a PIO→BitBang fallback without
//! DMA for baud rates supported by at least one backend.

use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::peripherals::{
    DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, DMA_CH4, DMA_CH5, DMA_CH6, DMA_CH7, DMA_CH8, DMA_CH9,
    DMA_CH10, DMA_CH11, DMA_CH12, DMA_CH13, DMA_CH14, DMA_CH15, I2C0, I2C1, PIO0, PIO1, PIO2, SPI0,
    SPI1, UART0, UART1,
};
use embassy_rp::pio::PioPin;
use embassy_rp::spi::{self, ClkPin, MisoPin, MosiPin};
use embassy_rp::{Peri, i2c, uart};

use alloc::boxed::Box;

use crate::bus::i2c::I2cBusHandle;
use crate::bus::i2c_bitbang::BitBangI2cBus;
use crate::bus::i2c_hardware::HardwareI2cBus;
use crate::bus::pio::{PioManager, gpio_base_for_pins, spi_program_instructions};
use crate::bus::spi::{SpiBusHandle, SpiError};
use crate::bus::spi_bitbang::BitBangSpiBus;
use crate::bus::spi_hardware::HardwareSpiBus;
use crate::bus::uart::UartBusHandle;
use crate::bus::uart_bitbang::BitBangUartBus;
use crate::bus::uart_hardware::HardwareUartBus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
/// Startup-time resource allocation error.
pub enum AllocatorError {
    /// The requested hardware peripheral, DMA channel or PIO state machine is
    /// already in use.
    Exhausted,
    /// The requested backend, frequency, or pin-role combination is invalid.
    InvalidConfiguration,
}

impl From<SpiError> for AllocatorError {
    fn from(_: SpiError) -> Self {
        Self::InvalidConfiguration
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// RP235x PIO instances
pub enum PioBlock {
    Block0 = 0,
    Block1 = 1,
    Block2 = 2,
}

impl PioBlock {
    /// Every board PIO instance.
    pub const ALL: [PioBlock; 3] = [PioBlock::Block0, PioBlock::Block1, PioBlock::Block2];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// RP235x PIO state machine index within a block.
pub enum Sm {
    /// First state machine in a block.
    Sm0 = 0,
    /// Second state machine in a block.
    Sm1 = 1,
    /// Third state machine in a block.
    Sm2 = 2,
    /// Fourth state machine in a block.
    Sm3 = 3,
}

impl Sm {
    /// All state-machine indices within a single PIO block.
    pub const ALL: [Sm; 4] = [Sm::Sm0, Sm::Sm1, Sm::Sm2, Sm::Sm3];
}

// ── sealed instance traits ───────────────────────────────────────────────
//
// These let the allocator dispatch to the right typed slot for a generic
// instance `I` *and* return a `Peri<'static, I>`, so the hardware bus can be
// constructed with the same generic `I` as its pins — no `panic!`
// "mismatched resource" arms, no unsafe transmutes.

mod private {
    pub trait Sealed {}
    impl Sealed for super::SPI0 {}
    impl Sealed for super::SPI1 {}
    impl Sealed for super::I2C0 {}
    impl Sealed for super::I2C1 {}
    impl Sealed for super::UART0 {}
    impl Sealed for super::UART1 {}
    impl Sealed for super::DMA_CH0 {}
    impl Sealed for super::DMA_CH1 {}
    impl Sealed for super::DMA_CH2 {}
    impl Sealed for super::DMA_CH3 {}
    impl Sealed for super::DMA_CH4 {}
    impl Sealed for super::DMA_CH5 {}
    impl Sealed for super::DMA_CH6 {}
    impl Sealed for super::DMA_CH7 {}
    impl Sealed for super::DMA_CH8 {}
    impl Sealed for super::DMA_CH9 {}
    impl Sealed for super::DMA_CH10 {}
    impl Sealed for super::DMA_CH11 {}
    impl Sealed for super::DMA_CH12 {}
    impl Sealed for super::DMA_CH13 {}
    impl Sealed for super::DMA_CH14 {}
    impl Sealed for super::DMA_CH15 {}
}

/// A hardware SPI instance the allocator can hand out.
pub trait SpiHw: spi::Instance + private::Sealed + Send + 'static {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>>;
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>);
}

/// A hardware I2C instance the allocator can hand out.
pub trait I2cHw: i2c::Instance + private::Sealed + Send + 'static {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>>;
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>);
}

/// A hardware UART instance the allocator can hand out.
pub trait UartHw: uart::Instance + private::Sealed + Send + 'static {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>>;
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>);
}

/// A DMA channel the allocator can hand out. Each RP235x DMA channel is a
/// distinct type; the caller specifies which channel(s) to use, and the
/// allocator tracks availability.
pub trait DmaChannel: ChannelInstance + private::Sealed + 'static {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>>;
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>);
}

// ── SPI peripheral impls ──

impl SpiHw for SPI0 {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
        alloc.spi0_peri.take()
    }
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
        alloc.spi0_peri = Some(peri);
    }
}

impl SpiHw for SPI1 {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
        alloc.spi1_peri.take()
    }
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
        alloc.spi1_peri = Some(peri);
    }
}

// ── I2C peripheral impls ──

impl I2cHw for I2C0 {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
        alloc.i2c0_peri.take()
    }
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
        alloc.i2c0_peri = Some(peri);
    }
}

impl I2cHw for I2C1 {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
        alloc.i2c1_peri.take()
    }
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
        alloc.i2c1_peri = Some(peri);
    }
}

// ── UART peripheral impls ──

impl UartHw for UART0 {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
        alloc.uart0_peri.take()
    }
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
        alloc.uart0_peri = Some(peri);
    }
}

impl UartHw for UART1 {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
        alloc.uart1_peri.take()
    }
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
        alloc.uart1_peri = Some(peri);
    }
}

// ── DMA channel impls ──

macro_rules! impl_dma_channel {
    ($($ch:ident => $field:ident),* $(,)?) => {
        $(
            impl DmaChannel for $ch {
                fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>> {
                    alloc.$field.take()
                }
                fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>) {
                    alloc.$field = Some(peri);
                }
            }
        )*
    };
}

impl_dma_channel! {
    DMA_CH0 => dma_ch0, DMA_CH1 => dma_ch1, DMA_CH2 => dma_ch2, DMA_CH3 => dma_ch3,
    DMA_CH4 => dma_ch4, DMA_CH5 => dma_ch5, DMA_CH6 => dma_ch6, DMA_CH7 => dma_ch7,
    DMA_CH8 => dma_ch8, DMA_CH9 => dma_ch9, DMA_CH10 => dma_ch10, DMA_CH11 => dma_ch11,
    DMA_CH12 => dma_ch12, DMA_CH13 => dma_ch13, DMA_CH14 => dma_ch14, DMA_CH15 => dma_ch15,
}

pub struct BusAllocator {
    spi0_peri: Option<Peri<'static, SPI0>>,
    spi1_peri: Option<Peri<'static, SPI1>>,
    i2c0_peri: Option<Peri<'static, I2C0>>,
    i2c1_peri: Option<Peri<'static, I2C1>>,
    uart0_peri: Option<Peri<'static, UART0>>,
    uart1_peri: Option<Peri<'static, UART1>>,
    dma_ch0: Option<Peri<'static, DMA_CH0>>,
    dma_ch1: Option<Peri<'static, DMA_CH1>>,
    dma_ch2: Option<Peri<'static, DMA_CH2>>,
    dma_ch3: Option<Peri<'static, DMA_CH3>>,
    dma_ch4: Option<Peri<'static, DMA_CH4>>,
    dma_ch5: Option<Peri<'static, DMA_CH5>>,
    dma_ch6: Option<Peri<'static, DMA_CH6>>,
    dma_ch7: Option<Peri<'static, DMA_CH7>>,
    dma_ch8: Option<Peri<'static, DMA_CH8>>,
    dma_ch9: Option<Peri<'static, DMA_CH9>>,
    dma_ch10: Option<Peri<'static, DMA_CH10>>,
    dma_ch11: Option<Peri<'static, DMA_CH11>>,
    dma_ch12: Option<Peri<'static, DMA_CH12>>,
    dma_ch13: Option<Peri<'static, DMA_CH13>>,
    dma_ch14: Option<Peri<'static, DMA_CH14>>,
    dma_ch15: Option<Peri<'static, DMA_CH15>>,
    pio_manager: PioManager,
}

/// Set of DMA channels the board hands to the allocator. Fields left as
/// `None` are not owned by the allocator and can't be dispensed.
pub struct DmaPool {
    pub ch0: Option<Peri<'static, DMA_CH0>>,
    pub ch1: Option<Peri<'static, DMA_CH1>>,
    pub ch2: Option<Peri<'static, DMA_CH2>>,
    pub ch3: Option<Peri<'static, DMA_CH3>>,
    pub ch4: Option<Peri<'static, DMA_CH4>>,
    pub ch5: Option<Peri<'static, DMA_CH5>>,
    pub ch6: Option<Peri<'static, DMA_CH6>>,
    pub ch7: Option<Peri<'static, DMA_CH7>>,
    pub ch8: Option<Peri<'static, DMA_CH8>>,
    pub ch9: Option<Peri<'static, DMA_CH9>>,
    pub ch10: Option<Peri<'static, DMA_CH10>>,
    pub ch11: Option<Peri<'static, DMA_CH11>>,
    pub ch12: Option<Peri<'static, DMA_CH12>>,
    pub ch13: Option<Peri<'static, DMA_CH13>>,
    pub ch14: Option<Peri<'static, DMA_CH14>>,
    pub ch15: Option<Peri<'static, DMA_CH15>>,
}

impl DmaPool {
    pub const fn none() -> Self {
        Self {
            ch0: None,
            ch1: None,
            ch2: None,
            ch3: None,
            ch4: None,
            ch5: None,
            ch6: None,
            ch7: None,
            ch8: None,
            ch9: None,
            ch10: None,
            ch11: None,
            ch12: None,
            ch13: None,
            ch14: None,
            ch15: None,
        }
    }
}

impl BusAllocator {
    /// Creates an allocator holding every provided peripheral.
    ///
    /// Supply `None` for any hardware instance the board does not want to expose
    /// to drivers. Every PIO block is mandatory because PIO state machines back
    /// several fallback buses.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use xpanse_api::bus::allocator::{BusAllocator, DmaPool};
    ///
    /// let p = embassy_rp::init(Default::default());
    /// let buses = BusAllocator::new(
    ///     Some(p.SPI0),
    ///     Some(p.SPI1),
    ///     Some(p.I2C0),
    ///     Some(p.I2C1),
    ///     Some(p.UART0),
    ///     Some(p.UART1),
    ///     DmaPool::none(),
    ///     p.PIO0,
    ///     p.PIO1,
    ///     p.PIO2,
    /// );
    /// ```
    pub fn new(
        spi0: Option<Peri<'static, SPI0>>,
        spi1: Option<Peri<'static, SPI1>>,
        i2c0: Option<Peri<'static, I2C0>>,
        i2c1: Option<Peri<'static, I2C1>>,
        uart0: Option<Peri<'static, UART0>>,
        uart1: Option<Peri<'static, UART1>>,
        dma: DmaPool,
        pio0: Peri<'static, PIO0>,
        pio1: Peri<'static, PIO1>,
        pio2: Peri<'static, PIO2>,
    ) -> Self {
        Self {
            spi0_peri: spi0,
            spi1_peri: spi1,
            i2c0_peri: i2c0,
            i2c1_peri: i2c1,
            uart0_peri: uart0,
            uart1_peri: uart1,
            dma_ch0: dma.ch0,
            dma_ch1: dma.ch1,
            dma_ch2: dma.ch2,
            dma_ch3: dma.ch3,
            dma_ch4: dma.ch4,
            dma_ch5: dma.ch5,
            dma_ch6: dma.ch6,
            dma_ch7: dma.ch7,
            dma_ch8: dma.ch8,
            dma_ch9: dma.ch9,
            dma_ch10: dma.ch10,
            dma_ch11: dma.ch11,
            dma_ch12: dma.ch12,
            dma_ch13: dma.ch13,
            dma_ch14: dma.ch14,
            dma_ch15: dma.ch15,
            pio_manager: PioManager::new(pio0, pio1, pio2),
        }
    }

    // ── SPI ──────────────────────────────────────────────────────────

    /// Requests exclusive access to one hardware SPI peripheral.
    ///
    /// Returns [`AllocatorError::Exhausted`] if the peripheral has already been
    /// handed out.
    pub fn request_spi_hardware<I: SpiHw>(&mut self) -> Result<Peri<'static, I>, AllocatorError> {
        I::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Returns a hardware SPI peripheral to the reusable pool.
    pub fn release_spi_hardware<I: SpiHw>(&mut self, peri: Peri<'static, I>) {
        I::return_peri(self, peri);
    }

    /// Requests exclusive access to one DMA channel.
    ///
    /// Returns [`AllocatorError::Exhausted`] if the channel has already been
    /// handed out.
    pub fn request_dma<C: DmaChannel>(&mut self) -> Result<Peri<'static, C>, AllocatorError> {
        C::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Returns a DMA channel to the reusable pool.
    pub fn release_dma<C: DmaChannel>(&mut self, peri: Peri<'static, C>) {
        C::return_peri(self, peri);
    }

    /// Builds a hardware SPI bus backed by DMA (truly async). Pin roles and
    /// instance are checked at compile time: `clk` must be a `ClkPin<I>`,
    /// `mosi` a `MosiPin<I>`, `miso` a `MisoPin<I>`. The DMA channels are
    /// pulled from the allocator's pool; the IRQ binding is the board's
    /// zero-sized `bind_interrupts!` type.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] for an unsupported SPI
    /// clock, or if `TxDma` and `RxDma` name the same DMA channel type. Returns
    /// [`AllocatorError::Exhausted`] if the SPI peripheral or either DMA channel
    /// is unavailable; in that case already-acquired resources are released.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use embassy_rp::spi;
    /// use xpanse_api::bus::allocator::BusAllocator;
    /// use xpanse_api::reexports::embassy_rp::{peripherals::*, interrupt::typelevel::Binding};
    ///
    /// embassy_rp::bind_interrupts!(struct Irqs {
    ///     DMA_CH0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
    ///     DMA_CH1 => embassy_rp::dma::InterruptHandler<DMA_CH1>;
    /// });
    ///
    /// fn make_spi(buses: &mut BusAllocator, p: SplitParts) -> SpiBusHandle {
    ///     buses.create_spi_hardware::<SPI0, DMA_CH0, DMA_CH1, _>(
    ///         p.gpio2, p.gpio4, p.gpio3, Irqs, spi::Config::default(),
    ///     )
    ///     .expect("SPI configured in range")
    /// }
    /// # use xpanse_api::bus::spi::SpiBusHandle;
    /// ```
    pub fn create_spi_hardware<I, TxDma, RxDma, Irq>(
        &mut self,
        clk: Peri<'static, impl ClkPin<I>>,
        mosi: Peri<'static, impl MosiPin<I>>,
        miso: Peri<'static, impl MisoPin<I>>,
        irq: Irq,
        config: spi::Config,
    ) -> Result<SpiBusHandle, AllocatorError>
    where
        I: SpiHw,
        TxDma: DmaChannel,
        RxDma: DmaChannel,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        HardwareSpiBus::<I>::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        if core::any::TypeId::of::<TxDma>() == core::any::TypeId::of::<RxDma>() {
            return Err(AllocatorError::InvalidConfiguration);
        }

        let peri = self.request_spi_hardware::<I>()?;
        let tx_dma = match self.request_dma::<TxDma>() {
            Ok(tx_dma) => tx_dma,
            Err(error) => {
                self.release_spi_hardware(peri);
                return Err(error);
            }
        };
        let rx_dma = match self.request_dma::<RxDma>() {
            Ok(rx_dma) => rx_dma,
            Err(error) => {
                self.release_dma(tx_dma);
                self.release_spi_hardware(peri);
                return Err(error);
            }
        };
        let bus = HardwareSpiBus::new(peri, clk, mosi, miso, tx_dma, rx_dma, irq, config)
            .expect("SPI configuration was validated before allocation");
        Ok(SpiBusHandle::new(
            Box::new(bus),
            crate::bus::spi::SpiBusVersion::Hardware,
        ))
    }

    /// Builds a PIO-backed SPI bus (async via DMA) on any free PIO state machine.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the clock frequency
    /// is unsupported, the pins span different GPIO banks, or the requested TX
    /// and RX DMA channels are the same type. Returns
    /// [`AllocatorError::Exhausted`] if no matching PIO state machine or DMA
    /// channel is free; acquired DMA channels are released on failure.
    pub fn create_spi_pio<TxDma, RxDma, Irq>(
        &mut self,
        clk: Peri<'static, impl PioPin>,
        mosi: Peri<'static, impl PioPin>,
        miso: Peri<'static, impl PioPin>,
        irq: Irq,
        config: spi::Config,
    ) -> Result<SpiBusHandle, AllocatorError>
    where
        TxDma: DmaChannel,
        RxDma: DmaChannel,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        crate::bus::spi_pio::PioSpiBus::<PIO0, 0>::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        if core::any::TypeId::of::<TxDma>() == core::any::TypeId::of::<RxDma>() {
            return Err(AllocatorError::InvalidConfiguration);
        }

        let gpio_base_high = gpio_base_for_pins(&[clk.pin(), mosi.pin(), miso.pin()])
            .ok_or(AllocatorError::InvalidConfiguration)?;
        let instructions = spi_program_instructions(&config);
        let (block, sm) = self
            .pio_manager
            .find_free_sm(gpio_base_high, instructions)
            .ok_or(AllocatorError::Exhausted)?;
        let tx_dma = self.request_dma::<TxDma>()?;
        let rx_dma = match self.request_dma::<RxDma>() {
            Ok(rx_dma) => rx_dma,
            Err(error) => {
                self.release_dma(tx_dma);
                return Err(error);
            }
        };
        Ok(self.pio_manager.build_spi_at(
            block,
            sm,
            gpio_base_high,
            instructions,
            clk,
            mosi,
            miso,
            tx_dma,
            rx_dma,
            irq,
            config,
        ))
    }

    /// Builds a bit-banged SPI bus using only GPIO.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] only for an SPI clock
    /// outside the bit-bang timing range.
    pub fn create_spi_bitbang(
        &mut self,
        clk: Peri<'static, impl embassy_rp::gpio::Pin>,
        mosi: Peri<'static, impl embassy_rp::gpio::Pin>,
        miso: Peri<'static, impl embassy_rp::gpio::Pin>,
        config: spi::Config,
    ) -> Result<SpiBusHandle, AllocatorError> {
        BitBangSpiBus::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        let bus = BitBangSpiBus::new(clk, mosi, miso, config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        Ok(SpiBusHandle::new(
            Box::new(bus),
            crate::bus::spi::SpiBusVersion::BitBang,
        ))
    }

    /// Builds a SPI bus that doesn't use hardware, preferring PIO then bit-bang.
    ///
    /// If PIO cannot be built, any
    /// PIO SM reservation is released before dropping to a bit-banged bus.
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the frequency is
    /// unsupported anywhere, or if the requested TX and RX DMA channels are the
    /// same type.
    pub fn create_spi_no_hardware<TxDma, RxDma, Irq>(
        &mut self,
        clk: Peri<'static, impl PioPin>,
        mosi: Peri<'static, impl PioPin>,
        miso: Peri<'static, impl PioPin>,
        irq: Irq,
        config: spi::Config,
    ) -> Result<SpiBusHandle, AllocatorError>
    where
        TxDma: DmaChannel,
        RxDma: DmaChannel,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        crate::bus::spi_pio::PioSpiBus::<PIO0, 0>::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        if core::any::TypeId::of::<TxDma>() == core::any::TypeId::of::<RxDma>() {
            return Err(AllocatorError::InvalidConfiguration);
        }

        let gpio_base_high = gpio_base_for_pins(&[clk.pin(), mosi.pin(), miso.pin()])
            .ok_or(AllocatorError::InvalidConfiguration);
        let instructions = spi_program_instructions(&config);
        let tx_dma = self.request_dma::<TxDma>();
        let rx_dma = self.request_dma::<RxDma>();

        match (gpio_base_high, tx_dma, rx_dma) {
            (Ok(gpio_base_high), Ok(tx_dma), Ok(rx_dma)) => {
                if let Ok((block, sm)) = self
                    .pio_manager
                    .find_free_sm(gpio_base_high, instructions)
                    .ok_or(AllocatorError::Exhausted)
                {
                    return Ok(self.pio_manager.build_spi_at(
                        block,
                        sm,
                        gpio_base_high,
                        instructions,
                        clk,
                        mosi,
                        miso,
                        tx_dma,
                        rx_dma,
                        irq,
                        config,
                    ));
                }
            }
            (_, tx_dma, rx_dma) => {
                if let Ok(tx_dma) = tx_dma {
                    self.release_dma(tx_dma);
                }

                if let Ok(rx_dma) = rx_dma {
                    self.release_dma(rx_dma);
                }
            }
        }

        self.create_spi_bitbang(clk, mosi, miso, config)
    }

    /// Builds a SPI bus, preferring hardware then PIO then bit-bang.
    ///
    /// Pins must implement both role-checked SPI traits and `PioPin` so a
    /// PIO fallback remains possible. If hardware SPI cannot be built, DMA
    /// channels are released before falling back. If PIO cannot be built, any
    /// PIO SM reservation is released before dropping to a bit-banged bus.
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the frequency is
    /// unsupported anywhere, or if the requested TX and RX DMA channels are the
    /// same type.
    pub fn create_spi<I, TxDma, RxDma, Irq>(
        &mut self,
        clk: Peri<'static, impl ClkPin<I> + PioPin>,
        mosi: Peri<'static, impl MosiPin<I> + PioPin>,
        miso: Peri<'static, impl MisoPin<I> + PioPin>,
        irq: Irq,
        config: spi::Config,
    ) -> Result<SpiBusHandle, AllocatorError>
    where
        I: SpiHw,
        TxDma: DmaChannel,
        RxDma: DmaChannel,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        HardwareSpiBus::<I>::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        if core::any::TypeId::of::<TxDma>() == core::any::TypeId::of::<RxDma>() {
            return Err(AllocatorError::InvalidConfiguration);
        }

        let peri = self.request_spi_hardware::<I>();
        let tx_dma = self.request_dma::<TxDma>();
        let rx_dma = self.request_dma::<RxDma>();

        match (peri, tx_dma, rx_dma) {
            (Ok(peri), Ok(tx_dma), Ok(rx_dma)) => {
                let bus = HardwareSpiBus::new(peri, clk, mosi, miso, tx_dma, rx_dma, irq, config)
                    .expect("SPI configuration was validated before allocation");
                Ok(SpiBusHandle::new(
                    Box::new(bus),
                    crate::bus::spi::SpiBusVersion::Hardware,
                ))
            }
            (peri, tx_dma, rx_dma) => {
                if let Ok(peri) = peri {
                    self.release_spi_hardware(peri);
                }
                if let Ok(tx_dma) = tx_dma {
                    self.release_dma(tx_dma);
                }
                if let Ok(rx_dma) = rx_dma {
                    self.release_dma(rx_dma);
                }

                self.create_spi_no_hardware::<TxDma, RxDma, Irq>(clk, mosi, miso, irq, config)
            }
        }
    }

    // ── I2C ──────────────────────────────────────────────────────────

    /// Requests exclusive access to one hardware I2C peripheral.
    ///
    /// Returns [`AllocatorError::Exhausted`] if the peripheral has already been
    /// handed out.
    pub fn request_i2c_hardware<I: I2cHw>(&mut self) -> Result<Peri<'static, I>, AllocatorError> {
        I::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Returns a hardware I2C peripheral to the reusable pool.
    pub fn release_i2c_hardware<I: I2cHw>(&mut self, peri: Peri<'static, I>) {
        I::return_peri(self, peri);
    }

    /// Builds a hardware I2C bus (async, interrupt-driven — no DMA needed).
    ///
    /// `scl` and `sda` are role-checked against `I` at compile time, and `irq`
    /// is the board's `bind_interrupts!` type for the I2C interrupt.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the frequency or
    /// derived clock dividers are out of range, or
    /// [`AllocatorError::Exhausted`] if the I2C peripheral is unavailable.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use embassy_rp::i2c;
    /// use xpanse_api::bus::allocator::BusAllocator;
    ///
    /// embassy_rp::bind_interrupts!(struct Irqs {
    ///     I2C0_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
    /// });
    ///
    /// let bus = buses.create_i2c_hardware::<embassy_rp::peripherals::I2C0, _>(
    ///     p.gpio0, p.gpio1, Irqs, i2c::Config::default(),
    /// )
    /// .expect("I2C configured in range");
    /// ```
    pub fn create_i2c_hardware<I, Irq>(
        &mut self,
        scl: Peri<'static, impl i2c::SclPin<I>>,
        sda: Peri<'static, impl i2c::SdaPin<I>>,
        irq: Irq,
        config: i2c::Config,
    ) -> Result<I2cBusHandle, AllocatorError>
    where
        I: I2cHw,
        Irq: Binding<I::Interrupt, i2c::InterruptHandler<I>> + 'static,
    {
        HardwareI2cBus::<I>::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        let peri = self.request_i2c_hardware::<I>()?;
        let bus = HardwareI2cBus::new(peri, scl, sda, irq, config)
            .expect("I2C configuration was validated before allocation");
        Ok(I2cBusHandle::new(
            Box::new(bus),
            crate::bus::i2c::I2cBusVersion::Hardware,
        ))
    }

    /// Builds a bit-banged I2C bus using two GPIO pins with open-drain
    /// capability. Frequencies outside the timer's range are rejected.
    /// `scl` and `sda` may be any GPIO pins; they are not role-checked.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] for a frequency above
    /// twice the timer tick rate or zero.
    pub fn create_i2c_bitbang(
        &mut self,
        scl: Peri<'static, impl embassy_rp::gpio::Pin>,
        sda: Peri<'static, impl embassy_rp::gpio::Pin>,
        frequency_hz: u32,
    ) -> Result<I2cBusHandle, AllocatorError> {
        let bus = BitBangI2cBus::new(scl, sda, frequency_hz)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        Ok(I2cBusHandle::new(
            Box::new(bus),
            crate::bus::i2c::I2cBusVersion::BitBang,
        ))
    }

    // ── UART ─────────────────────────────────────────────────────────

    /// Requests exclusive access to one hardware UART peripheral.
    ///
    /// Returns [`AllocatorError::Exhausted`] if the peripheral has already been
    /// handed out.
    pub fn request_uart_hardware<I: UartHw>(&mut self) -> Result<Peri<'static, I>, AllocatorError> {
        I::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Returns a hardware UART peripheral to the reusable pool.
    pub fn release_uart_hardware<I: UartHw>(&mut self, peri: Peri<'static, I>) {
        I::return_peri(self, peri);
    }

    /// Builds a hardware (DMA) UART bus.
    ///
    /// `tx` and `rx` are role-checked against `I` at compile time. The DMA
    /// channels are pulled from the allocator's pool.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] for an unsupported
    /// baud rate or if `TxDma` and `RxDma` name the same channel type. Returns
    /// [`AllocatorError::Exhausted`] if the UART peripheral or a DMA channel is
    /// unavailable; already-acquired resources are released on failure.
    pub fn create_uart_hardware<I, TxDma, RxDma, Irq>(
        &mut self,
        tx: Peri<'static, impl uart::TxPin<I>>,
        rx: Peri<'static, impl uart::RxPin<I>>,
        irq: Irq,
        config: uart::Config,
    ) -> Result<UartBusHandle, AllocatorError>
    where
        I: UartHw,
        TxDma: DmaChannel,
        RxDma: DmaChannel,
        Irq: Binding<I::Interrupt, uart::InterruptHandler<I>>
            + Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        HardwareUartBus::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        if core::any::TypeId::of::<TxDma>() == core::any::TypeId::of::<RxDma>() {
            return Err(AllocatorError::InvalidConfiguration);
        }

        let peri = self.request_uart_hardware::<I>()?;
        let tx_dma = match self.request_dma::<TxDma>() {
            Ok(tx_dma) => tx_dma,
            Err(error) => {
                self.release_uart_hardware(peri);
                return Err(error);
            }
        };
        let rx_dma = match self.request_dma::<RxDma>() {
            Ok(rx_dma) => rx_dma,
            Err(error) => {
                self.release_dma(tx_dma);
                self.release_uart_hardware(peri);
                return Err(error);
            }
        };
        let bus = HardwareUartBus::new(peri, tx, rx, irq, tx_dma, rx_dma, config)
            .expect("UART configuration was validated before allocation");
        Ok(UartBusHandle::new(
            Box::new(bus),
            crate::bus::uart::UartBusVersion::Hardware,
        ))
    }

    /// Builds a PIO-backed UART bus on any two free state machines of one block.
    ///
    /// PIO UART uses FIFO polling and needs no DMA.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the baud rate is
    /// unsupported or `tx` and `rx` are not in the same GPIO bank. Returns
    /// [`AllocatorError::Exhausted`] if no PIO state machine pair is free.
    pub fn create_uart_pio(
        &mut self,
        tx: Peri<'static, impl PioPin>,
        rx: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> Result<UartBusHandle, AllocatorError> {
        crate::bus::uart_pio::PioUartBus::<PIO0, 0, 1>::validate_baud(baud_rate)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        let gpio_base_high =
            gpio_base_for_pins(&[tx.pin()]).ok_or(AllocatorError::InvalidConfiguration)?;
        if gpio_base_for_pins(&[rx.pin()]) != Some(gpio_base_high) {
            return Err(AllocatorError::InvalidConfiguration);
        }
        self.pio_manager
            .build_uart_pio(tx, rx, baud_rate)
            .ok_or(AllocatorError::Exhausted)
    }

    /// Builds a bit-banged UART bus using only GPIO at a representable baud.
    ///
    /// 8-N-1 framing, idle-high TX line, one-byte reads.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] for a baud rate of zero
    /// or above the timer's maximum.
    pub fn create_uart_bitbang(
        &mut self,
        tx: Peri<'static, impl embassy_rp::gpio::Pin>,
        rx: Peri<'static, impl embassy_rp::gpio::Pin>,
        baud_rate: u32,
    ) -> Result<UartBusHandle, AllocatorError> {
        BitBangUartBus::validate_baud(baud_rate)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        let bus = BitBangUartBus::new(tx, rx, baud_rate)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;
        Ok(UartBusHandle::new(
            Box::new(bus),
            crate::bus::uart::UartBusVersion::BitBang,
        ))
    }

    /// Builds a UART bus, preferring PIO then bit-bang.
    ///
    /// Hardware UART requires DMA — request it explicitly with
    /// [`create_uart_hardware`](Self::create_uart_hardware) or
    /// [`BusAllocator::create_uart`](crate::bus::allocator::BusAllocator::create_uart)(Self::create_uart).
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the baud rate is
    /// unsupported or `tx` and `rx` are not in the same GPIO bank. Returns
    /// [`AllocatorError::Exhausted`] if PIO state machines are free but all
    /// fallbacks ultimately fail.
    pub fn create_uart_no_hardware(
        &mut self,
        tx: Peri<'static, impl PioPin>,
        rx: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> Result<UartBusHandle, AllocatorError> {
        crate::bus::uart_pio::PioUartBus::<PIO0, 0, 1>::validate_baud(baud_rate)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;

        let tx_base = gpio_base_for_pins(&[tx.pin()]);
        let rx_base = gpio_base_for_pins(&[rx.pin()]);
        if let (Some(gpio_base_high), true) = (tx_base, tx_base == rx_base)
            && let Some((block, sm_tx, sm_rx)) = self
                .pio_manager
                .find_free_sm_pair(gpio_base_high, crate::bus::pio::PIO_UART_INSTRUCTIONS)
        {
            return Ok(self.pio_manager.build_uart_at(
                block,
                sm_tx,
                sm_rx,
                gpio_base_high,
                crate::bus::pio::PIO_UART_INSTRUCTIONS,
                tx,
                rx,
                baud_rate,
            ));
        }
        self.create_uart_bitbang(tx, rx, baud_rate)
    }

    /// Builds a UART bus, preferring hardware then PIO then bit-bang.
    ///
    /// Pins must implement both role-checked UART traits and `PioPin` so a
    /// PIO fallback remains possible. On the hardware failure path, any
    /// acquired UART peripheral or DMA channel is released before the fallback
    /// is attempted.
    ///
    /// # Errors
    ///
    /// Returns [`AllocatorError::InvalidConfiguration`] if the baud rate is
    /// unsupported or `TxDma` and `RxDma` name the same channel type. Returns
    /// [`AllocatorError::Exhausted`] if all three backends are unavailable.
    pub fn create_uart<I, TxDma, RxDma, Irq>(
        &mut self,
        tx: Peri<'static, impl uart::TxPin<I> + PioPin>,
        rx: Peri<'static, impl uart::RxPin<I> + PioPin>,
        irq: Irq,
        config: uart::Config,
    ) -> Result<UartBusHandle, AllocatorError>
    where
        I: UartHw,
        TxDma: DmaChannel,
        RxDma: DmaChannel,
        Irq: Binding<I::Interrupt, uart::InterruptHandler<I>>
            + Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        HardwareUartBus::validate_config(&config)
            .map_err(|_| AllocatorError::InvalidConfiguration)?;

        if core::any::TypeId::of::<TxDma>() == core::any::TypeId::of::<RxDma>() {
            return Err(AllocatorError::InvalidConfiguration);
        }

        let peri = self.request_uart_hardware::<I>();
        let tx_dma = self.request_dma::<TxDma>();
        let rx_dma = self.request_dma::<RxDma>();

        match (peri, tx_dma, rx_dma) {
            // Success path: We get direct ownership of the unwrapped values
            (Ok(peri), Ok(tx_dma), Ok(rx_dma)) => {
                let bus = HardwareUartBus::new(peri, tx, rx, irq, tx_dma, rx_dma, config)
                    .expect("UART configuration was validated before allocation");

                Ok(UartBusHandle::new(
                    Box::new(bus),
                    crate::bus::uart::UartBusVersion::Hardware,
                ))
            }

            // Failure path: At least one is an Err.
            // We re-bind the original Results to these variables and clean up.
            (peri, tx_dma, rx_dma) => {
                if let Ok(peri) = peri {
                    self.release_uart_hardware(peri);
                }
                if let Ok(tx_dma) = tx_dma {
                    self.release_dma(tx_dma);
                }
                if let Ok(rx_dma) = rx_dma {
                    self.release_dma(rx_dma);
                }

                self.create_uart_no_hardware(tx, rx, config.baudrate)
            }
        }
    }

    // ── PIO ─────────────────────────────────────────────────────────

    /// Hands out one free PIO state machine on any block, together with the
    /// block's `Common` handle. Drivers that load custom PIO programs use this,
    /// then call [`with_pio!`](crate::with_pio!) to dispatch over the erased
    /// block/SM types.
    ///
    /// The `Common` borrow is only valid while the returned [`crate::bus::pio::PioAccess`] is
    /// alive (i.e. while you hold `&mut BusAllocator`). Programs loaded via
    /// `Common` produce `'static` handles, so a driver can load a program,
    /// configure the SM, and then keep the `LoadedProgram` + `StateMachine`
    /// after the borrow ends.
    ///
    /// Returns `None` if every state machine on every block is in use, the pin
    /// is not a PIO-capable GPIO, or 32 instructions have already been reserved.
    pub fn request_pio<P: PioPin>(
        &mut self,
        pin: &Peri<'static, P>,
    ) -> Option<crate::bus::pio::PioAccess<'_>> {
        self.pio_manager.request_pio(pin)
    }
}
