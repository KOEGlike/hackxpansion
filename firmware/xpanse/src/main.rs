#![no_std]
#![no_main]

extern crate alloc;

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::multicore::{Stack, spawn_core1};
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use xpanse::{app_core::app_core_task, resource_split::*, split_resources, ui_core::ui_core_task};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Hackxpansion"),
    embassy_rp::binary_info::rp_program_description!(c"Firmware for Hackxpansion"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    unsafe {
        embedded_alloc::init!(HEAP, 1024);
    }

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(unwrap!(app_core_task(
                    r.gpio_bank_0,
                    r.gpio_bank_1,
                    r.gpio_bank_2,
                    r.gpio_bank_3,
                    r.i2c_pins,
                    r.remaining_peris
                )))
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(unwrap!(ui_core_task(r.display))));
}
