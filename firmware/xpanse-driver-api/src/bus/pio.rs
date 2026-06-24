//! PIO resource manager.
//!
//! Owns the three PIO blocks and hands out state machines on demand. Unlike the
//! previous design, a `PioSlot` keeps the `Common` handle, all four state
//! machines and the IRQ handles alive for the lifetime of the slot, so SMs are
//! never orphaned and can be allocated independently up to the full 12 SMs.

use alloc::boxed::Box;

use embassy_rp::dma::{self, ChannelInstance};
use embassy_rp::interrupt::typelevel::Binding;
use embassy_rp::peripherals::{PIO0, PIO1, PIO2};
use embassy_rp::pio::{Common, Instance, Irq, IrqFlags, Pio, PioPin, StateMachine};
use embassy_rp::spi;
use embassy_rp::Peri;

use crate::bus::allocator::{PioBlock, Sm};
use crate::bus::spi::{DynSpiBusCombined, SpiBusHandle, SpiBusVersion};
use crate::bus::spi_pio::PioSpiBus;
use crate::bus::uart::{DynUartBus, UartBusHandle, UartBusVersion};
use crate::bus::uart_pio::PioUartBus;

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
}

impl<PIO: Instance + 'static> PioSlot<PIO> {
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
        }
    }

    fn has_sm(&self, sm: Sm) -> bool {
        match sm {
            Sm::Sm0 => self.sm0.is_some(),
            Sm::Sm1 => self.sm1.is_some(),
            Sm::Sm2 => self.sm2.is_some(),
            Sm::Sm3 => self.sm3.is_some(),
        }
    }

    fn build_spi<TxDma, RxDma, Irq>(
        &mut self,
        sm: Sm,
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
                    Some(sm) => Some(Box::new(PioSpiBus::new(
                        &mut self.common,
                        sm,
                        clk,
                        mosi,
                        miso,
                        tx_dma,
                        rx_dma,
                        irq,
                        config,
                    )) as Box<dyn DynSpiBusCombined>),
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
        tx_pin: Peri<'static, impl PioPin>,
        rx_pin: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> Option<Box<dyn DynUartBus>> {
        macro_rules! pair {
            ($txf:ident, $rxf:ident) => {
                match (self.$txf.take(), self.$rxf.take()) {
                    (Some(t), Some(r)) => Some(Box::new(PioUartBus::new(
                        &mut self.common,
                        t,
                        r,
                        tx_pin,
                        rx_pin,
                        baud_rate,
                    )) as Box<dyn DynUartBus>),
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
                .and_then(|slot| slot.build_spi(sm, clk, mosi, miso, tx_dma, rx_dma, irq, config))
                .expect("PIO SM allocation invariant: SM was not free"),
            PioBlock::Block1 => self
                .pio1
                .as_mut()
                .and_then(|slot| slot.build_spi(sm, clk, mosi, miso, tx_dma, rx_dma, irq, config))
                .expect("PIO SM allocation invariant: SM was not free"),
            PioBlock::Block2 => self
                .pio2
                .as_mut()
                .and_then(|slot| slot.build_spi(sm, clk, mosi, miso, tx_dma, rx_dma, irq, config))
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
        tx_pin: Peri<'static, impl PioPin>,
        rx_pin: Peri<'static, impl PioPin>,
        baud_rate: u32,
    ) -> UartBusHandle {
        let bus = match block {
            PioBlock::Block0 => self
                .pio0
                .as_mut()
                .and_then(|slot| slot.build_uart(sm_tx, sm_rx, tx_pin, rx_pin, baud_rate))
                .expect("PIO SM allocation invariant: SM pair was not free"),
            PioBlock::Block1 => self
                .pio1
                .as_mut()
                .and_then(|slot| slot.build_uart(sm_tx, sm_rx, tx_pin, rx_pin, baud_rate))
                .expect("PIO SM allocation invariant: SM pair was not free"),
            PioBlock::Block2 => self
                .pio2
                .as_mut()
                .and_then(|slot| slot.build_uart(sm_tx, sm_rx, tx_pin, rx_pin, baud_rate))
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
        let (block, sm_tx, sm_rx) = self.find_free_sm_pair()?;
        Some(self.build_uart_at(block, sm_tx, sm_rx, tx_pin, rx_pin, baud_rate))
    }

    pub(crate) fn find_free_sm(&self) -> Option<(PioBlock, Sm)> {
        if let Some(slot) = &self.pio0 {
            for sm in Sm::ALL {
                if slot.has_sm(sm) {
                    return Some((PioBlock::Block0, sm));
                }
            }
        }
        if let Some(slot) = &self.pio1 {
            for sm in Sm::ALL {
                if slot.has_sm(sm) {
                    return Some((PioBlock::Block1, sm));
                }
            }
        }
        if let Some(slot) = &self.pio2 {
            for sm in Sm::ALL {
                if slot.has_sm(sm) {
                    return Some((PioBlock::Block2, sm));
                }
            }
        }
        None
    }

    pub(crate) fn find_free_sm_pair(&self) -> Option<(PioBlock, Sm, Sm)> {
        if let Some(slot) = &self.pio0
            && let Some((a, b)) = two_free_sms(slot)
        {
            return Some((PioBlock::Block0, a, b));
        }
        if let Some(slot) = &self.pio1
            && let Some((a, b)) = two_free_sms(slot)
        {
            return Some((PioBlock::Block1, a, b));
        }
        if let Some(slot) = &self.pio2
            && let Some((a, b)) = two_free_sms(slot)
        {
            return Some((PioBlock::Block2, a, b));
        }
        None
    }
}

fn two_free_sms<PIO: Instance + 'static>(slot: &PioSlot<PIO>) -> Option<(Sm, Sm)> {
    let mut free = Sm::ALL
        .into_iter()
        .filter(|sm| slot.has_sm(*sm));
    match (free.next(), free.next()) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}
