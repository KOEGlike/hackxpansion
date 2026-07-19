//! PIO resource manager.
//!
//! Owns the three PIO blocks and hands out state machines on demand. Unlike the
//! previous design, a `PioSlot` keeps the `Common` handle, all four state
//! machines and the IRQ handles alive for the lifetime of the slot, so SMs are
//! never orphaned and can be allocated independently up to the full 12 SMs.

use alloc::boxed::Box;

use embassy_rp::Peri;
use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::peripherals::{PIO0, PIO1, PIO2};
use embassy_rp::pio::{Common, Instance, Irq, IrqFlags, Pio, PioPin, StateMachine};
use embassy_rp::spi;

use crate::bus::allocator::{PioBlock, Sm};
use crate::bus::spi::{DynSpiBusCombined, SpiBusHandle, SpiBusVersion};
use crate::bus::spi_pio::PioSpiBus;
use crate::bus::uart::{DynUartBus, UartBusHandle, UartBusVersion};
use crate::bus::uart_pio::PioUartBus;

pub(crate) const PIO_UART_INSTRUCTIONS: u8 = 14;

pub(crate) fn spi_program_instructions(config: &spi::Config) -> u8 {
    match config.phase {
        spi::Phase::CaptureOnFirstTransition => 2,
        spi::Phase::CaptureOnSecondTransition => 3,
    }
}

pub(crate) fn gpio_base_for_pins(pins: &[u8]) -> Option<bool> {
    if pins.iter().all(|pin| *pin < 32) {
        Some(false)
    } else if pins.iter().all(|pin| *pin >= 16) {
        Some(true)
    } else {
        None
    }
}

/// A `StateMachine` with the SM number erased at runtime.
///
/// Drivers that load custom PIO programs match on this enum to recover the
/// const-generic `StateMachine<'static, PIO, N>` they need.
pub enum AnyStateMachine<'d, PIO: Instance + 'static> {
    Sm0(StateMachine<'d, PIO, 0>),
    Sm1(StateMachine<'d, PIO, 1>),
    Sm2(StateMachine<'d, PIO, 2>),
    Sm3(StateMachine<'d, PIO, 3>),
}

/// A handle returned by [`BusAllocator::request_pio`](crate::bus::allocator::BusAllocator::request_pio)
/// that gives a driver temporary access to one block's `Common` handle plus a
/// free `StateMachine`.
///
/// The `Common` borrow is only valid for the lifetime of the handle (which is
/// tied to `&mut BusAllocator`).  Programs loaded via `Common` return handles
/// with `'static` lifetime (because the `Common` inside `BusAllocator` is
/// `Common<'static, PIO>`), so a driver can load a program, configure the SM,
/// and then keep the `LoadedProgram` + `StateMachine` after the borrow ends.
#[must_use = "custom PIO access reserves its block for the rest of the boot"]
pub enum PioAccess<'a> {
    Block0 {
        common: &'a mut Common<'static, PIO0>,
        sm: AnyStateMachine<'static, PIO0>,
    },
    Block1 {
        common: &'a mut Common<'static, PIO1>,
        sm: AnyStateMachine<'static, PIO1>,
    },
    Block2 {
        common: &'a mut Common<'static, PIO2>,
        sm: AnyStateMachine<'static, PIO2>,
    },
}

embassy_rp::bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO1>;
    PIO2_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO2>;
});

/// A single PIO block: the shared `Common` handle plus all four state machines
/// and IRQ handles, kept alive together so dropping one piece never decrements
/// the PIO's user counter prematurely.
struct PioSlot<PIO: Instance + 'static> {
    common: Common<'static, PIO>,
    sm0: Option<StateMachine<'static, PIO, 0>>,
    sm1: Option<StateMachine<'static, PIO, 1>>,
    sm2: Option<StateMachine<'static, PIO, 2>>,
    sm3: Option<StateMachine<'static, PIO, 3>>,
    _irq_flags: IrqFlags<'static, PIO>,
    _irq0: Irq<'static, PIO, 0>,
    _irq1: Irq<'static, PIO, 1>,
    _irq2: Irq<'static, PIO, 2>,
    _irq3: Irq<'static, PIO, 3>,
    gpio_base_high: Option<bool>,
    reserved_instructions: u8,
}

impl<PIO: Instance + Send + 'static> PioSlot<PIO> {
    fn new(pio: Pio<'static, PIO>) -> Self {
        let Pio {
            common,
            irq_flags,
            irq0,
            irq1,
            irq2,
            irq3,
            sm0,
            sm1,
            sm2,
            sm3,
            ..
        } = pio;
        Self {
            common,
            sm0: Some(sm0),
            sm1: Some(sm1),
            sm2: Some(sm2),
            sm3: Some(sm3),
            _irq_flags: irq_flags,
            _irq0: irq0,
            _irq1: irq1,
            _irq2: irq2,
            _irq3: irq3,
            gpio_base_high: None,
            reserved_instructions: 0,
        }
    }

    fn can_reserve(&self, gpio_base_high: bool, instructions: u8) -> bool {
        self.gpio_base_high
            .is_none_or(|current| current == gpio_base_high)
            && self.reserved_instructions.saturating_add(instructions) <= 32
    }

    fn reserve(&mut self, gpio_base_high: bool, instructions: u8) {
        debug_assert!(self.can_reserve(gpio_base_high, instructions));
        self.gpio_base_high = Some(gpio_base_high);
        self.reserved_instructions += instructions;
    }

    fn has_sm(&self, sm: Sm) -> bool {
        match sm {
            Sm::Sm0 => self.sm0.is_some(),
            Sm::Sm1 => self.sm1.is_some(),
            Sm::Sm2 => self.sm2.is_some(),
            Sm::Sm3 => self.sm3.is_some(),
        }
    }

    fn take_any_sm(&mut self, sm: Sm) -> Option<AnyStateMachine<'static, PIO>> {
        match sm {
            Sm::Sm0 => self.sm0.take().map(AnyStateMachine::Sm0),
            Sm::Sm1 => self.sm1.take().map(AnyStateMachine::Sm1),
            Sm::Sm2 => self.sm2.take().map(AnyStateMachine::Sm2),
            Sm::Sm3 => self.sm3.take().map(AnyStateMachine::Sm3),
        }
    }

    fn build_spi<TxDma, RxDma, Irq>(
        &mut self,
        sm: Sm,
        gpio_base_high: bool,
        instructions: u8,
        clk: Peri<'static, impl PioPin>,
        mosi: Peri<'static, impl PioPin>,
        miso: Peri<'static, impl PioPin>,
        tx_dma: Peri<'static, TxDma>,
        rx_dma: Peri<'static, RxDma>,
        irq: Irq,
        config: spi::Config,
    ) -> Option<Box<dyn DynSpiBusCombined>>
    where
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        macro_rules! take {
            ($field:ident) => {
                match self.$field.take() {
                    Some(sm) => {
                        self.reserve(gpio_base_high, instructions);
                        Some(Box::new(
                            PioSpiBus::new(
                                &mut self.common,
                                sm,
                                clk,
                                mosi,
                                miso,
                                tx_dma,
                                rx_dma,
                                irq,
                                config,
                            )
                            .expect("PIO SPI configuration was validated before allocation"),
                        ) as Box<dyn DynSpiBusCombined>)
                    }
                    None => None,
                }
            };
        }

        match sm {
            Sm::Sm0 => take!(sm0),
            Sm::Sm1 => take!(sm1),
            Sm::Sm2 => take!(sm2),
            Sm::Sm3 => take!(sm3),
        }
    }

    fn build_uart(
        &mut self,
        sm_tx: Sm,
        sm_rx: Sm,
        gpio_base_high: bool,
        instructions: u8,
        tx_pin: Peri<'static, impl PioPin>,
        rx_pin: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> Option<Box<dyn DynUartBus>> {
        macro_rules! pair {
            ($txf:ident, $rxf:ident) => {
                match (self.$txf.take(), self.$rxf.take()) {
                    (Some(t), Some(r)) => {
                        self.reserve(gpio_base_high, instructions);
                        Some(Box::new(
                            PioUartBus::new(&mut self.common, t, r, tx_pin, rx_pin, baud_rate)
                                .expect("PIO UART baud was validated before allocation"),
                        ) as Box<dyn DynUartBus>)
                    }
                    (Some(t), None) => {
                        self.$txf = Some(t);
                        None
                    }
                    (None, _) => None,
                }
            };
        }

        match (sm_tx, sm_rx) {
            (Sm::Sm0, Sm::Sm1) => pair!(sm0, sm1),
            (Sm::Sm0, Sm::Sm2) => pair!(sm0, sm2),
            (Sm::Sm0, Sm::Sm3) => pair!(sm0, sm3),
            (Sm::Sm1, Sm::Sm0) => pair!(sm1, sm0),
            (Sm::Sm1, Sm::Sm2) => pair!(sm1, sm2),
            (Sm::Sm1, Sm::Sm3) => pair!(sm1, sm3),
            (Sm::Sm2, Sm::Sm0) => pair!(sm2, sm0),
            (Sm::Sm2, Sm::Sm1) => pair!(sm2, sm1),
            (Sm::Sm2, Sm::Sm3) => pair!(sm2, sm3),
            (Sm::Sm3, Sm::Sm0) => pair!(sm3, sm0),
            (Sm::Sm3, Sm::Sm1) => pair!(sm3, sm1),
            (Sm::Sm3, Sm::Sm2) => pair!(sm3, sm2),
            // Equal pairs are never produced by the allocator (it picks two
            // *distinct* free SMs), so this arm is unreachable in correct use.
            _ => None,
        }
    }
}

pub struct PioManager {
    pio0: Option<PioSlot<PIO0>>,
    pio1: Option<PioSlot<PIO1>>,
    pio2: Option<PioSlot<PIO2>>,
}

impl PioManager {
    pub fn new(
        pio0: Peri<'static, PIO0>,
        pio1: Peri<'static, PIO1>,
        pio2: Peri<'static, PIO2>,
    ) -> Self {
        Self {
            pio0: Some(PioSlot::new(Pio::new(pio0, PioIrqs))),
            pio1: Some(PioSlot::new(Pio::new(pio1, PioIrqs))),
            pio2: Some(PioSlot::new(Pio::new(pio2, PioIrqs))),
        }
    }

    /// Build a PIO SPI bus on a specific (block, sm). The SM must be free —
    /// i.e. obtained from [`find_free_sm`] in the same synchronous sequence.
    pub(crate) fn build_spi_at<TxDma, RxDma, Irq>(
        &mut self,
        block: PioBlock,
        sm: Sm,
        gpio_base_high: bool,
        instructions: u8,
        clk: Peri<'static, impl PioPin>,
        mosi: Peri<'static, impl PioPin>,
        miso: Peri<'static, impl PioPin>,
        tx_dma: Peri<'static, TxDma>,
        rx_dma: Peri<'static, RxDma>,
        irq: Irq,
        config: spi::Config,
    ) -> SpiBusHandle
    where
        TxDma: ChannelInstance,
        RxDma: ChannelInstance,
        Irq: Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
            + Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
            + 'static,
    {
        let bus = match block {
            PioBlock::Block0 => self
                .pio0
                .as_mut()
                .and_then(|slot| {
                    slot.build_spi(
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
                    )
                })
                .expect("PIO SM allocation invariant: SM was not free"),
            PioBlock::Block1 => self
                .pio1
                .as_mut()
                .and_then(|slot| {
                    slot.build_spi(
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
                    )
                })
                .expect("PIO SM allocation invariant: SM was not free"),
            PioBlock::Block2 => self
                .pio2
                .as_mut()
                .and_then(|slot| {
                    slot.build_spi(
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
                    )
                })
                .expect("PIO SM allocation invariant: SM was not free"),
        };
        SpiBusHandle::new(bus, SpiBusVersion::Pio)
    }

    /// Build a PIO UART bus on a specific (block, sm_tx, sm_rx). The SMs must be
    /// free — obtained from [`find_free_sm_pair`] in the same sequence.
    pub(crate) fn build_uart_at(
        &mut self,
        block: PioBlock,
        sm_tx: Sm,
        sm_rx: Sm,
        gpio_base_high: bool,
        instructions: u8,
        tx_pin: Peri<'static, impl PioPin>,
        rx_pin: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> UartBusHandle {
        let bus = match block {
            PioBlock::Block0 => self
                .pio0
                .as_mut()
                .and_then(|slot| {
                    slot.build_uart(
                        sm_tx,
                        sm_rx,
                        gpio_base_high,
                        instructions,
                        tx_pin,
                        rx_pin,
                        baud_rate,
                    )
                })
                .expect("PIO SM allocation invariant: SM pair was not free"),
            PioBlock::Block1 => self
                .pio1
                .as_mut()
                .and_then(|slot| {
                    slot.build_uart(
                        sm_tx,
                        sm_rx,
                        gpio_base_high,
                        instructions,
                        tx_pin,
                        rx_pin,
                        baud_rate,
                    )
                })
                .expect("PIO SM allocation invariant: SM pair was not free"),
            PioBlock::Block2 => self
                .pio2
                .as_mut()
                .and_then(|slot| {
                    slot.build_uart(
                        sm_tx,
                        sm_rx,
                        gpio_base_high,
                        instructions,
                        tx_pin,
                        rx_pin,
                        baud_rate,
                    )
                })
                .expect("PIO SM allocation invariant: SM pair was not free"),
        };
        UartBusHandle::new(bus, UartBusVersion::Pio)
    }

    /// Build a PIO-backed UART bus on any two free state machines of a single
    /// block (one for TX, one for RX).
    pub fn build_uart_pio(
        &mut self,
        tx_pin: Peri<'static, impl PioPin>,
        rx_pin: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> Option<UartBusHandle> {
        let gpio_base_high = gpio_base_for_pins(&[tx_pin.pin()])?;
        if gpio_base_for_pins(&[rx_pin.pin()])? != gpio_base_high {
            return None;
        }
        let (block, sm_tx, sm_rx) =
            self.find_free_sm_pair(gpio_base_high, PIO_UART_INSTRUCTIONS)?;
        Some(self.build_uart_at(
            block,
            sm_tx,
            sm_rx,
            gpio_base_high,
            PIO_UART_INSTRUCTIONS,
            tx_pin,
            rx_pin,
            baud_rate,
        ))
    }

    /// Hand out one free state machine on any block, together with the block's
    /// `Common` handle.  Drivers use this to load custom PIO programs.
    pub fn request_pio(&mut self, pin: &Peri<'static, impl PioPin>) -> Option<PioAccess<'_>> {
        let gpio_base_high = gpio_base_for_pins(&[pin.pin()])?;
        let (block, sm) = self.find_free_sm(gpio_base_high, 32)?;
        match block {
            PioBlock::Block0 => {
                let slot = self.pio0.as_mut()?;
                let sm = slot.take_any_sm(sm)?;
                slot.reserve(gpio_base_high, 32);
                Some(PioAccess::Block0 {
                    common: &mut slot.common,
                    sm,
                })
            }
            PioBlock::Block1 => {
                let slot = self.pio1.as_mut()?;
                let sm = slot.take_any_sm(sm)?;
                slot.reserve(gpio_base_high, 32);
                Some(PioAccess::Block1 {
                    common: &mut slot.common,
                    sm,
                })
            }
            PioBlock::Block2 => {
                let slot = self.pio2.as_mut()?;
                let sm = slot.take_any_sm(sm)?;
                slot.reserve(gpio_base_high, 32);
                Some(PioAccess::Block2 {
                    common: &mut slot.common,
                    sm,
                })
            }
        }
    }

    pub(crate) fn find_free_sm(
        &self,
        gpio_base_high: bool,
        instructions: u8,
    ) -> Option<(PioBlock, Sm)> {
        if let Some(slot) = &self.pio0 {
            for sm in Sm::ALL {
                if slot.has_sm(sm) && slot.can_reserve(gpio_base_high, instructions) {
                    return Some((PioBlock::Block0, sm));
                }
            }
        }
        if let Some(slot) = &self.pio1 {
            for sm in Sm::ALL {
                if slot.has_sm(sm) && slot.can_reserve(gpio_base_high, instructions) {
                    return Some((PioBlock::Block1, sm));
                }
            }
        }
        if let Some(slot) = &self.pio2 {
            for sm in Sm::ALL {
                if slot.has_sm(sm) && slot.can_reserve(gpio_base_high, instructions) {
                    return Some((PioBlock::Block2, sm));
                }
            }
        }
        None
    }

    pub(crate) fn find_free_sm_pair(
        &self,
        gpio_base_high: bool,
        instructions: u8,
    ) -> Option<(PioBlock, Sm, Sm)> {
        if let Some(slot) = &self.pio0
            && slot.can_reserve(gpio_base_high, instructions)
            && let Some((a, b)) = two_free_sms(slot)
        {
            return Some((PioBlock::Block0, a, b));
        }
        if let Some(slot) = &self.pio1
            && slot.can_reserve(gpio_base_high, instructions)
            && let Some((a, b)) = two_free_sms(slot)
        {
            return Some((PioBlock::Block1, a, b));
        }
        if let Some(slot) = &self.pio2
            && slot.can_reserve(gpio_base_high, instructions)
            && let Some((a, b)) = two_free_sms(slot)
        {
            return Some((PioBlock::Block2, a, b));
        }
        None
    }
}

fn two_free_sms<PIO: Instance + Send + 'static>(slot: &PioSlot<PIO>) -> Option<(Sm, Sm)> {
    let mut free = Sm::ALL.into_iter().filter(|sm| slot.has_sm(*sm));
    match (free.next(), free.next()) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

/// Dispatch over a [`PioAccess`] handle, recovering the typed `Common` and
/// `StateMachine` so a driver can load a custom PIO program.
///
/// The driver provides a **generic** function/closure that works for any
/// `PIO: Instance` and `const N: usize`.  The macro matches all 12
/// (block × SM) arms and calls the function in the one that matches.
///
/// # Example
///
/// ```ignore
/// use embassy_rp::pio::{Common, Instance, StateMachine};
///
/// fn my_init<PIO: Instance, const N: usize>(
///     common: &mut Common<'static, PIO>,
///     sm: StateMachine<'static, PIO, N>,
/// ) -> MyDriver<PIO, N> {
///     let program = MyProgram::new(common);
///     MyDriver::new(common, sm, &program)
/// }
///
/// let handle = allocator.request_pio(&pin)?;
/// let driver = with_pio!(handle, common, sm, my_init(common, sm));
/// ```
#[macro_export]
macro_rules! with_pio {
    ($handle:expr, $common:ident, $sm:ident, $body:expr) => {
        match $handle {
            $crate::bus::pio::PioAccess::Block0 { common, sm } => {
                let $common = common;
                match sm {
                    $crate::bus::pio::AnyStateMachine::Sm0($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm1($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm2($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm3($sm) => $body,
                }
            }
            $crate::bus::pio::PioAccess::Block1 { common, sm } => {
                let $common = common;
                match sm {
                    $crate::bus::pio::AnyStateMachine::Sm0($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm1($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm2($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm3($sm) => $body,
                }
            }
            $crate::bus::pio::PioAccess::Block2 { common, sm } => {
                let $common = common;
                match sm {
                    $crate::bus::pio::AnyStateMachine::Sm0($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm1($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm2($sm) => $body,
                    $crate::bus::pio::AnyStateMachine::Sm3($sm) => $body,
                }
            }
        }
    };
}
