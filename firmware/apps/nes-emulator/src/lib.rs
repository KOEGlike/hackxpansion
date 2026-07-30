#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use core::{cell::Cell, future::Future, pin::Pin};

use embassy_futures::yield_now;
use embassy_time::{Duration, Instant, Ticker};
use runes::{
    apu::{APU, Speaker},
    cartridge::{BankType, Cartridge, MirrorType},
    controller::{InputPoller, stdctl},
    mapper::{self, Mapper},
    memory::{CPUMemory, PPUMemory},
    mos6502,
    ppu::{self, PPU},
    utils::{Read, Write},
};
use xpanse_api::{
    app::App,
    interfaces::{
        buttons::{A, B, Button, Down, Left, Right, Up, X, Y},
        video::{Rgb565FrameBuffer, Rgb565FrameSession, Rgb565Pixel},
    },
    registry::{Registry, ResourceLease},
};

include!(concat!(env!("OUT_DIR"), "/rom.rs"));

const NES_WIDTH: u16 = 256;
const NES_HEIGHT: u16 = 240;
const FRAME_INTERVAL: Duration = Duration::from_micros(16_667);
const DISPLAY_FRAME_DIVISOR: u32 = 2;
const STATS_FRAME_INTERVAL: u32 = 300;
const SRAM_SIZE: usize = 0x2000;
const CHR_BANK_SIZE: usize = 0x2000;

const RGB_COLORS: [u32; 64] = [
    0x666666, 0x002a88, 0x1412a7, 0x3b00a4, 0x5c007e, 0x6e0040, 0x6c0600, 0x561d00, 0x333500,
    0x0b4800, 0x005200, 0x004f08, 0x00404d, 0x000000, 0x000000, 0x000000, 0xadadad, 0x155fd9,
    0x4240ff, 0x7527fe, 0xa01acc, 0xb71e7b, 0xb53120, 0x994e00, 0x6b6d00, 0x388700, 0x0c9300,
    0x008f32, 0x007c8d, 0x000000, 0x000000, 0x000000, 0xfffeff, 0x64b0ff, 0x9290ff, 0xc676ff,
    0xf36aff, 0xfe6ecc, 0xfe8170, 0xea9e22, 0xbcbe00, 0x88d800, 0x5ce430, 0x45e082, 0x48cdde,
    0x4f4f4f, 0x000000, 0x000000, 0xfffeff, 0xc0dfff, 0xd3d2ff, 0xe8c8ff, 0xfbc2ff, 0xfec4ea,
    0xfeccc5, 0xf7d8a5, 0xe4e594, 0xcfef96, 0xbdf4ab, 0xb3f3cc, 0xb5ebf2, 0xb8b8b8, 0x000000,
    0x000000,
];
const NES_PALETTE: [Rgb565Pixel; 64] = rgb565_palette(RGB_COLORS);

type AppButton<R> = ResourceLease<Box<dyn Button<R>>>;
type NesResources = (
    Box<dyn Button<Up>>,
    Box<dyn Button<Down>>,
    Box<dyn Button<Left>>,
    Box<dyn Button<Right>>,
    Box<dyn Button<A>>,
    Box<dyn Button<B>>,
    Box<dyn Button<X>>,
    Box<dyn Button<Y>>,
    Rgb565FrameBuffer,
);

pub struct NesEmulatorApp {
    up: AppButton<Up>,
    down: AppButton<Down>,
    left: AppButton<Left>,
    right: AppButton<Right>,
    a: AppButton<A>,
    b: AppButton<B>,
    select: AppButton<X>,
    start: AppButton<Y>,
    frame_buffer: ResourceLease<Rgb565FrameBuffer>,
}

impl App for NesEmulatorApp {
    const NAME: &'static str = "NES Emulator";

    fn can_run(registry: &Registry) -> bool {
        !ROM.is_empty() && registry.has_resource_set::<NesResources>()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let (up, down, left, right, a, b, select, start, frame_buffer) =
            registry.take_resource_set::<NesResources>()?;
        Some(Self {
            up,
            down,
            left,
            right,
            a,
            b,
            select,
            start,
            frame_buffer,
        })
    }

    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let rom = match ParsedRom::parse(ROM) {
                Ok(rom) => rom,
                Err(error) => {
                    defmt::error!("NES: invalid ROM: {}", error);
                    return;
                }
            };

            let input = NesInput {
                up: self.up.resource().as_ref(),
                down: self.down.resource().as_ref(),
                left: self.left.resource().as_ref(),
                right: self.right.resource().as_ref(),
                a: self.a.resource().as_ref(),
                b: self.b.resource().as_ref(),
                select: self.select.resource().as_ref(),
                start: self.start.resource().as_ref(),
            };
            let joystick = stdctl::Joystick::new(&input);
            let frame_ready = Cell::new(false);
            let frame_buffer = match self
                .frame_buffer
                .resource_mut()
                .start(NES_WIDTH, NES_HEIGHT)
            {
                Ok(frame_buffer) => frame_buffer,
                Err(error) => {
                    defmt::error!("NES: failed to start framebuffer: {}", error);
                    return;
                }
            };
            let mut screen = NesScreen {
                frame_buffer,
                frame_ready: &frame_ready,
                frame_number: 0,
            };
            let mut speaker = SilentSpeaker;

            let cart = EmbeddedCart::new(rom.prg_rom, rom.chr_rom, rom.mirror);
            let mut mapper: Box<dyn Mapper> = match rom.mapper {
                0 | 2 => Box::new(mapper::Mapper2::new(cart)),
                1 => Box::new(mapper::Mapper1::new(cart)),
                4 => Box::new(mapper::Mapper4::new(cart)),
                mapper => {
                    defmt::error!("NES: mapper {} is not supported by RuNES", mapper);
                    return;
                }
            };

            let mapper_ref = mapper::RefMapper::new(mapper.as_mut());
            let mut cpu = mos6502::CPU::new(CPUMemory::new(&mapper_ref, Some(&joystick), None));
            let mut ppu = PPU::new(PPUMemory::new(&mapper_ref), &mut screen);
            let mut apu = APU::new(&mut speaker);
            let cpu_ptr = &mut cpu as *mut mos6502::CPU;
            cpu.mem.bus.attach(cpu_ptr, &mut ppu, &mut apu);
            cpu.powerup();

            let mut frame_ticker = Ticker::every(FRAME_INTERVAL);
            let mut stats_started = Instant::now();
            let mut stats_frames = 0_u32;
            loop {
                loop {
                    if cpu.cycle == 0 {
                        break;
                    }
                    cpu.mem.bus.tick();
                }
                cpu.step();

                if frame_ready.replace(false) {
                    if input.exit_pressed() {
                        break;
                    }
                    stats_frames += 1;
                    if stats_frames == STATS_FRAME_INTERVAL {
                        defmt::info!(
                            "NES: {} frames in {} ms",
                            stats_frames,
                            stats_started.elapsed().as_millis()
                        );
                        stats_started = Instant::now();
                        stats_frames = 0;
                    }
                    // A ticker that has fallen behind completes immediately, so it cannot
                    // guarantee that the sibling display future gets polled.
                    yield_now().await;
                    frame_ticker.next().await;
                }
            }
        })
    }

    fn release(self, registry: &mut Registry) {
        registry.return_resource(self.up);
        registry.return_resource(self.down);
        registry.return_resource(self.left);
        registry.return_resource(self.right);
        registry.return_resource(self.a);
        registry.return_resource(self.b);
        registry.return_resource(self.select);
        registry.return_resource(self.start);
        registry.return_resource(self.frame_buffer);
    }
}

struct NesInput<'a> {
    up: &'a dyn Button<Up>,
    down: &'a dyn Button<Down>,
    left: &'a dyn Button<Left>,
    right: &'a dyn Button<Right>,
    a: &'a dyn Button<A>,
    b: &'a dyn Button<B>,
    select: &'a dyn Button<X>,
    start: &'a dyn Button<Y>,
}

impl NesInput<'_> {
    fn exit_pressed(&self) -> bool {
        self.up.is_pressed() && self.down.is_pressed()
    }
}

impl InputPoller for NesInput<'_> {
    fn poll(&self) -> u8 {
        let mut state = stdctl::NULL;
        state |= if self.a.is_pressed() { stdctl::A } else { 0 };
        state |= if self.b.is_pressed() { stdctl::B } else { 0 };
        state |= if self.select.is_pressed() {
            stdctl::SELECT
        } else {
            0
        };
        state |= if self.start.is_pressed() {
            stdctl::START
        } else {
            0
        };
        state |= if self.up.is_pressed() { stdctl::UP } else { 0 };
        state |= if self.down.is_pressed() {
            stdctl::DOWN
        } else {
            0
        };
        state |= if self.left.is_pressed() {
            stdctl::LEFT
        } else {
            0
        };
        state |= if self.right.is_pressed() {
            stdctl::RIGHT
        } else {
            0
        };
        state
    }
}

struct NesScreen<'a> {
    frame_buffer: Rgb565FrameSession<'a>,
    frame_ready: &'a Cell<bool>,
    frame_number: u32,
}

impl ppu::Screen for NesScreen<'_> {
    fn put(&mut self, x: u8, y: u8, color: u8) {
        self.frame_buffer.set_pixel(
            u16::from(x),
            u16::from(y),
            NES_PALETTE[usize::from(color & 0x3f)],
        );
    }

    fn render(&mut self) {}

    fn frame(&mut self) {
        self.frame_number = self.frame_number.wrapping_add(1);
        if self.frame_number.is_multiple_of(DISPLAY_FRAME_DIVISOR) {
            self.frame_buffer.present();
        }
        self.frame_ready.set(true);
    }
}

struct SilentSpeaker;

impl Speaker for SilentSpeaker {
    fn queue(&mut self, _sample: i16) {}
}

#[derive(Clone, Copy, defmt::Format)]
enum RomError {
    MissingHeader,
    InvalidMagic,
    Nes2,
    MissingPrg,
    Truncated,
}

struct ParsedRom {
    prg_rom: &'static [u8],
    chr_rom: Vec<u8>,
    mirror: MirrorType,
    mapper: u8,
}

impl ParsedRom {
    fn parse(data: &'static [u8]) -> Result<Self, RomError> {
        let header = data.get(..16).ok_or(RomError::MissingHeader)?;
        if header[..4] != *b"NES\x1a" {
            return Err(RomError::InvalidMagic);
        }
        if header[7] & 0x0c == 0x08 {
            return Err(RomError::Nes2);
        }

        let prg_len = usize::from(header[4]) * 0x4000;
        if prg_len == 0 {
            return Err(RomError::MissingPrg);
        }
        let chr_len = usize::from(header[5]) * CHR_BANK_SIZE;
        let trainer_len = if header[6] & 0x04 != 0 { 512 } else { 0 };
        let prg_start = 16 + trainer_len;
        let chr_start = prg_start + prg_len;
        let rom_end = chr_start + chr_len;
        if data.len() < rom_end {
            return Err(RomError::Truncated);
        }

        let mirror = if header[6] & 0x08 != 0 {
            MirrorType::Four
        } else if header[6] & 0x01 != 0 {
            MirrorType::Vertical
        } else {
            MirrorType::Horizontal
        };
        let mapper = (header[7] & 0xf0) | (header[6] >> 4);
        let chr_rom = if chr_len == 0 {
            vec![0; CHR_BANK_SIZE]
        } else {
            data[chr_start..rom_end].to_vec()
        };

        Ok(Self {
            prg_rom: &data[prg_start..chr_start],
            chr_rom,
            mirror,
            mapper,
        })
    }
}

struct EmbeddedCart {
    prg_rom: &'static [u8],
    chr_rom: Vec<u8>,
    sram: Vec<u8>,
    mirror: MirrorType,
}

impl EmbeddedCart {
    fn new(prg_rom: &'static [u8], chr_rom: Vec<u8>, mirror: MirrorType) -> Self {
        Self {
            prg_rom,
            chr_rom,
            sram: vec![0; SRAM_SIZE],
            mirror,
        }
    }
}

impl Cartridge for EmbeddedCart {
    fn get_size(&self, kind: BankType) -> usize {
        match kind {
            BankType::PrgRom => self.prg_rom.len(),
            BankType::ChrRom => self.chr_rom.len(),
            BankType::Sram => self.sram.len(),
        }
    }

    fn get_bank<'a>(&self, base: usize, size: usize, kind: BankType) -> &'a [u8] {
        let bank = match kind {
            BankType::PrgRom => self.prg_rom,
            BankType::ChrRom => self.chr_rom.as_slice(),
            BankType::Sram => self.sram.as_slice(),
        };
        let end = if size == 0 { bank.len() } else { base + size };
        let bank = &bank[base..end];
        // RuNES' Cartridge trait uses mapper-owned slices with an unconstrained lifetime.
        unsafe { &*(bank as *const [u8]) }
    }

    fn get_bank_mut<'a>(&mut self, base: usize, size: usize, kind: BankType) -> &'a mut [u8] {
        let bank = match kind {
            BankType::PrgRom => panic!("RuNES attempted to mutate PRG ROM"),
            BankType::ChrRom => self.chr_rom.as_mut_slice(),
            BankType::Sram => self.sram.as_mut_slice(),
        };
        let end = if size == 0 { bank.len() } else { base + size };
        let bank = &mut bank[base..end];
        // RuNES stores these slices in its mapper after the cartridge call returns.
        unsafe { &mut *(bank as *mut [u8]) }
    }

    fn get_mirror_type(&self) -> MirrorType {
        self.mirror
    }

    fn set_mirror_type(&mut self, mirror: MirrorType) {
        self.mirror = mirror;
    }

    fn load(&mut self, _reader: &mut dyn Read) -> bool {
        false
    }

    fn save(&self, _writer: &mut dyn Write) -> bool {
        false
    }

    fn load_sram(&mut self, reader: &mut dyn Read) -> bool {
        reader.read(&mut self.sram) == Some(self.sram.len())
    }

    fn save_sram(&self, writer: &mut dyn Write) -> bool {
        writer.write(&self.sram) == Some(self.sram.len())
    }
}

const fn rgb565_palette(colors: [u32; 64]) -> [Rgb565Pixel; 64] {
    let mut palette = [Rgb565Pixel(0); 64];
    let mut index = 0;
    while index < colors.len() {
        let color = colors[index];
        palette[index] = Rgb565Pixel(
            (((color >> 19) & 0x1f) << 11 | ((color >> 10) & 0x3f) << 5 | ((color >> 3) & 0x1f))
                as u16,
        );
        index += 1;
    }
    palette
}
