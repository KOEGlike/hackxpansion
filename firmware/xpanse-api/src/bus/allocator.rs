//! Hardware bus allocator.
//!
//! The allocator hands out peripherals (SPI/I2C/UART), PIO state machines and
//! DMA channels to drivers. The API is designed so that the most common ways to
//! misconfigure hardware are caught at compile time:
//!
//! * Pin roles can't be swapped — `clk`/`mosi`/`miso` and `tx`/`rx` are
//!   distinguished by trait bounds (`ClkPin<I>`, `MosiPin<I>`, …). A pin only
//!   implements the role trait for its actual function, so passing `miso` where
//!   `clk` is expected is a type error.
//! * Pins can't be paired with the wrong peripheral instance — the role traits
//!   are parameterised by the instance (`ClkPin<SPI0>` vs `ClkPin<SPI1>`), so an
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
//!   combination the allocator returns is unbuildable.
//!
//! Async SPI (hardware and PIO) requires DMA. Because embassy-rp models each
//! DMA channel as a distinct type, the caller specifies the channel types
//! (`TxDma`, `RxDma`) and provides the board's IRQ binding; the allocator
//! dispenses the matching `Peri` tokens from its pool. PIO UART uses FIFO
//! polling (no DMA), so [`create_uart`] is available as an infallible
//! PIO→BitBang fallback without DMA.

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
use crate::bus::pio::PioManager;
use crate::bus::spi::SpiBusHandle;
use crate::bus::spi_bitbang::BitBangSpiBus;
use crate::bus::spi_hardware::HardwareSpiBus;
use crate::bus::uart::UartBusHandle;
use crate::bus::uart_bitbang::BitBangUartBus;
use crate::bus::uart_hardware::HardwareUartBus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum AllocatorError {
    /// The requested hardware peripheral, DMA channel or PIO state machine is
    /// already in use.
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PioBlock {
    Block0 = 0,
    Block1 = 1,
    Block2 = 2,
}

impl PioBlock {
    pub const ALL: [PioBlock; 3] = [PioBlock::Block0, PioBlock::Block1, PioBlock::Block2];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm {
    Sm0 = 0,
    Sm1 = 1,
    Sm2 = 2,
    Sm3 = 3,
}

impl Sm {
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
pub trait SpiHw: spi::Instance + private::Sealed + 'static {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>>;
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>);
}

/// A hardware I2C instance the allocator can hand out.
pub trait I2cHw: i2c::Instance + private::Sealed + 'static {
    fn take_peri(alloc: &mut BusAllocator) -> Option<Peri<'static, Self>>;
    fn return_peri(alloc: &mut BusAllocator, peri: Peri<'static, Self>);
}

/// A hardware UART instance the allocator can hand out.
pub trait UartHw: uart::Instance + private::Sealed + 'static {
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

/// The set of DMA channels the board hands to the allocator. Fields left as
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
    /// No DMA channels — for boards that don't use async SPI/UART hardware.
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

    /// Take the hardware SPI peripheral for instance `I`. Returns `Exhausted`
    /// if it has already been handed out.
    pub fn request_spi_hardware<I: SpiHw>(&mut self) -> Result<Peri<'static, I>, AllocatorError> {
        I::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Return a hardware SPI peripheral to the pool.
    pub fn release_spi_hardware<I: SpiHw>(&mut self, peri: Peri<'static, I>) {
        I::return_peri(self, peri);
    }

    /// Take a DMA channel from the pool. Returns `Exhausted` if it has already
    /// been handed out.
    pub fn request_dma<C: DmaChannel>(&mut self) -> Result<Peri<'static, C>, AllocatorError> {
        C::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Return a DMA channel to the pool.
    pub fn release_dma<C: DmaChannel>(&mut self, peri: Peri<'static, C>) {
        C::return_peri(self, peri);
    }

    /// Build a hardware SPI bus backed by DMA (truly async). Pin roles and
    /// instance are checked at compile time: `clk` must be a `ClkPin<I>`,
    /// `mosi` a `MosiPin<I>`, `miso` a `MisoPin<I>`. The DMA channels are
    /// pulled from the allocator's pool; the IRQ binding is the board's
    /// zero-sized `bind_interrupts!` type.
    pub fn create_spi_hardware<I, TxDma, RxDma, Irq>(
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
        let peri = self.request_spi_hardware::<I>()?;
        let tx_dma = self.request_dma::<TxDma>()?;
        let rx_dma = self.request_dma::<RxDma>()?;
        let bus = HardwareSpiBus::new(peri, clk, mosi, miso, tx_dma, rx_dma, irq, config);
        Ok(SpiBusHandle::new(
            Box::new(bus),
            crate::bus::spi::SpiBusVersion::Hardware,
        ))
    }

    /// Build a PIO-backed SPI bus (async via DMA) on any free PIO state machine.
    pub fn create_spi_pio<I, TxDma, RxDma, Irq>(
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
        let tx_dma = self.request_dma::<TxDma>()?;
        let rx_dma = self.request_dma::<RxDma>()?;
        let (block, sm) = self
            .pio_manager
            .find_free_sm()
            .ok_or(AllocatorError::Exhausted)?;
        Ok(self
            .pio_manager
            .build_spi_at(block, sm, clk, mosi, miso, tx_dma, rx_dma, irq, config))
    }

    /// Build a bit-banged SPI bus (always available — only needs GPIO, no DMA).
    pub fn create_spi_bitbang<I: SpiHw>(
        &mut self,
        clk: Peri<'static, impl ClkPin<I> + PioPin>,
        mosi: Peri<'static, impl MosiPin<I> + PioPin>,
        miso: Peri<'static, impl MisoPin<I> + PioPin>,
        config: spi::Config,
    ) -> SpiBusHandle {
        let bus = BitBangSpiBus::new(clk, mosi, miso, config.frequency);
        SpiBusHandle::new(Box::new(bus), crate::bus::spi::SpiBusVersion::BitBang)
    }

    // ── I2C ──────────────────────────────────────────────────────────

    /// Take the hardware I2C peripheral for instance `I`. There is no PIO or
    /// bit-bang fallback for I2C yet — request it explicitly.
    pub fn request_i2c_hardware<I: I2cHw>(&mut self) -> Result<Peri<'static, I>, AllocatorError> {
        I::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Return a hardware I2C peripheral to the pool.
    pub fn release_i2c_hardware<I: I2cHw>(&mut self, peri: Peri<'static, I>) {
        I::return_peri(self, peri);
    }

    /// Build a hardware I2C bus (async, interrupt-driven — no DMA needed).
    /// `scl`/`sda` are role-checked against `I` at compile time. The IRQ
    /// binding is the board's `bind_interrupts!` type for the I2C interrupt.
    pub fn create_i2c_hardware<I, Irq>(
        &mut self,
        scl: Peri<'static, impl i2c::SclPin<I> + PioPin>,
        sda: Peri<'static, impl i2c::SdaPin<I> + PioPin>,
        irq: Irq,
        config: i2c::Config,
    ) -> Result<I2cBusHandle, AllocatorError>
    where
        I: I2cHw,
        Irq: Binding<I::Interrupt, i2c::InterruptHandler<I>> + 'static,
    {
        let peri = self.request_i2c_hardware::<I>()?;
        let bus = HardwareI2cBus::new(peri, scl, sda, irq, config);
        Ok(I2cBusHandle::new(
            Box::new(bus),
            crate::bus::i2c::I2cBusVersion::Hardware,
        ))
    }

    /// Build a bit-banged I2C bus (always available — only needs two GPIO
    /// pins with open-drain capability, no hardware I2C peripheral or DMA).
    /// `scl`/`sda` are role-checked against `I` at compile time.
    pub fn create_i2c_bitbang<I: I2cHw>(
        &mut self,
        scl: Peri<'static, impl i2c::SclPin<I> + PioPin>,
        sda: Peri<'static, impl i2c::SdaPin<I> + PioPin>,
        frequency_hz: u32,
    ) -> I2cBusHandle {
        let bus = BitBangI2cBus::new(scl, sda, frequency_hz);
        I2cBusHandle::new(Box::new(bus), crate::bus::i2c::I2cBusVersion::BitBang)
    }

    // ── UART ─────────────────────────────────────────────────────────

    /// Take the hardware UART peripheral for instance `I`.
    pub fn request_uart_hardware<I: UartHw>(&mut self) -> Result<Peri<'static, I>, AllocatorError> {
        I::take_peri(self).ok_or(AllocatorError::Exhausted)
    }

    /// Return a hardware UART peripheral to the pool.
    pub fn release_uart_hardware<I: UartHw>(&mut self, peri: Peri<'static, I>) {
        I::return_peri(self, peri);
    }

    /// Build a hardware (DMA) UART bus. `tx`/`rx` are role-checked against `I`
    /// at compile time. The DMA channels are pulled from the allocator's pool.
    pub fn create_uart_hardware<I, TxDma, RxDma, Irq>(
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
        let peri = self.request_uart_hardware::<I>()?;
        let tx_dma = self.request_dma::<TxDma>()?;
        let rx_dma = self.request_dma::<RxDma>()?;
        let bus = HardwareUartBus::new(peri, tx, rx, irq, tx_dma, rx_dma, config);
        Ok(UartBusHandle::new(
            Box::new(bus),
            crate::bus::uart::UartBusVersion::Hardware,
        ))
    }

    /// Build a PIO-backed UART bus on any two free state machines of one block.
    /// PIO UART uses FIFO polling (no DMA required).
    pub fn create_uart_pio<I: UartHw>(
        &mut self,
        tx: Peri<'static, impl uart::TxPin<I> + PioPin>,
        rx: Peri<'static, impl uart::RxPin<I> + PioPin>,
        baud_rate: u32,
    ) -> Result<UartBusHandle, AllocatorError> {
        self.pio_manager
            .build_uart_pio(tx, rx, baud_rate)
            .ok_or(AllocatorError::Exhausted)
    }

    /// Build a bit-banged UART bus (always available — only needs GPIO).
    pub fn create_uart_bitbang<I: UartHw>(
        &mut self,
        tx: Peri<'static, impl uart::TxPin<I> + PioPin>,
        rx: Peri<'static, impl uart::RxPin<I> + PioPin>,
        baud_rate: u32,
    ) -> UartBusHandle {
        let bus = BitBangUartBus::new(tx, rx, baud_rate);
        UartBusHandle::new(Box::new(bus), crate::bus::uart::UartBusVersion::BitBang)
    }

    /// Build a UART bus, preferring PIO then bit-bang. Never fails. Hardware
    /// UART requires DMA — request it explicitly with
    /// [`create_uart_hardware`](Self::create_uart_hardware).
    pub fn create_uart<I: UartHw>(
        &mut self,
        tx: Peri<'static, impl uart::TxPin<I> + PioPin>,
        rx: Peri<'static, impl uart::RxPin<I> + PioPin>,
        baud_rate: u32,
    ) -> UartBusHandle {
        if let Some((block, sm_tx, sm_rx)) = self.pio_manager.find_free_sm_pair() {
            return self
                .pio_manager
                .build_uart_at(block, sm_tx, sm_rx, tx, rx, baud_rate);
        }
        let bus = BitBangUartBus::new(tx, rx, baud_rate);
        UartBusHandle::new(Box::new(bus), crate::bus::uart::UartBusVersion::BitBang)
    }

    // ── PIO ─────────────────────────────────────────────────────────

    /// Hand out one free PIO state machine on any block, together with the
    /// block's `Common` handle.  Drivers that load custom PIO programs use
    /// this, then call [`with_pio!`](crate::with_pio!) to dispatch over the
    /// erased block/SM types.
    ///
    /// The `Common` borrow is only valid while the returned `PioAccess` is
    /// alive (i.e. while you hold `&mut BusAllocator`).  Programs loaded via
    /// `Common` produce `'static` handles, so a driver can load a program,
    /// configure the SM, and keep the results after the borrow ends.
    pub fn request_pio(&mut self) -> Option<crate::bus::pio::PioAccess<'_>> {
        self.pio_manager.request_pio()
    }
}
