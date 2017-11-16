use display::Display;
use memory::{Memory, BlockMemory};
use std::cell::RefCell;
use std::ops::Deref;

pub struct Gpu<'a> {
    mode: Mode,
    mode_clock: u32,
    vram: BlockMemory,
    sprites: BlockMemory,
    io: &'a RefCell<BlockMemory>,
    display: Display<'a>,
}

impl<'a> Gpu<'a> {
    pub fn new(display: Display<'a>, io: &'a RefCell<BlockMemory>) -> Gpu<'a> {
        Gpu {
            mode: Mode::HorizontalBlank,
            mode_clock: 0,
            vram: BlockMemory::new(0x2000),
            sprites: BlockMemory::new(0xA0),
            io,
            display,
        }
    }

    pub fn get_vram(&self) -> &Memory {
        &self.vram
    }

    pub fn get_vram_mut(&mut self) -> &mut Memory {
        &mut self.vram
    }

    pub fn get_sprites(&self) -> &Memory {
        &self.sprites
    }

    pub fn get_sprites_mut(&mut self) -> &mut Memory {
        &mut self.sprites
    }

    pub fn step(&mut self, cycles: u8) {
        let cycles = cycles as u32;
        self.mode_clock += cycles;
        match self.mode {
            Mode::ScanlineOam => {
                if self.mode_clock >= SCANLINE_OAM_TIME {
                    self.mode_clock %= SCANLINE_OAM_TIME;
                    self.mode = Mode::ScanlineVram;
                }
            }
            Mode::ScanlineVram => {
                if self.mode_clock >= SCANLINE_VRAM_TIME {
                    self.mode_clock %= SCANLINE_VRAM_TIME;
                    self.mode = Mode::HorizontalBlank;
                    self.render_scanline();
                }
            }
            Mode::HorizontalBlank => {
                if self.mode_clock >= HORIZONTAL_BLANK_TIME {
                    self.mode_clock %= HORIZONTAL_BLANK_TIME;
                    let new_line = self.increment_current_line();
                    if new_line >= DIM_Y - 1 {
                        self.mode = Mode::VerticalBlank;
                        self.render_screen();
                    } else {
                        self.mode = Mode::ScanlineOam;
                    }
                }
            }
            Mode::VerticalBlank => {
                if self.mode_clock >= VERTICAL_BLANK_TIME {
                    self.mode_clock %= VERTICAL_BLANK_TIME;
                    self.mode = Mode::ScanlineOam;
                    self.reset_current_line();
                }
            }
        }
    }

    fn get_color(vram: &BlockMemory, io: &BlockMemory, tile: u16, x: u16, y: u16) -> u8 {
        let bit = 1 << (7 - x);
        let mut offset = tile * 0x10;
        offset += y as u16 * 0x2;
        let low_bit = (vram.read_byte(offset) & bit) != 0;
        let high_bit = (vram.read_byte(offset + 1) & bit) != 0;
        let color = low_bit as u8 + high_bit as u8 * 2;
        let palette = io.read_byte(0x47);
        palette >> (color * 2) & 3
    }

    fn render_scanline(&mut self) {
        let mut map_offset = if Self::bg_on(&self.io.borrow()) {
            OFFSET_TILE_MAP_1
        } else {
            OFFSET_TILE_MAP_0
        };
        map_offset += ((self.get_current_line() as u16 + self.scy()) & 0xFF) >> 3;
        let mut line_offset = self.scx() >> 3;
        let y = (self.get_current_line() as u16 + self.scy()) & 0x7;
        let mut x = self.scx() & 0x7;
        let mut tile = self.vram.read_byte(map_offset + line_offset) as u16;
        if Self::bg_on(&self.io.borrow()) && tile < 128 {
            tile += 256;
        }

        let current_line = self.get_current_line();
        let display_line = self.display.get_line_mut(current_line);
        for i in 0..DIM_X {
            let color = Self::get_color(&self.vram, &self.io.borrow(), tile, x, y);
            display_line[i] = color;
            x = x % 8;
            if x == 0 {
                line_offset = (line_offset + 1) & 0x1F;
                tile = self.vram.read_byte(map_offset + line_offset) as u16;
                if Self::bg_on(&self.io.borrow()) && tile < 128 {
                    tile += 256;
                }
            }
        }
    }

    fn render_screen(&mut self) {
        self.display.redraw();
    }

    fn scx(&self) -> u16 {
        self.io.borrow().read_byte(0x43) as u16
    }

    fn scy(&self) -> u16 {
        self.io.borrow().read_byte(0x42) as u16
    }

    fn increment_current_line(&mut self) -> u8 {
        let current_line = self.io.borrow_mut().read_byte(0x44) + 1;
        self.io.borrow_mut().write_byte(0x44, current_line);
        current_line
    }

    fn get_current_line(&self) -> u8 {
        self.io.borrow().read_byte(0x44)
    }

    fn reset_current_line(&mut self) {
        self.io.borrow_mut().write_byte(0x44, 0);
    }

    fn get_gpu_control_register_static(io: &BlockMemory) -> u8 {
        io.read_byte(0x40)
    }

    fn get_gpu_control_register(&self) -> u8 {
        Self::get_gpu_control_register_static(self.io.borrow().deref())
    }

    fn bg_on(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register_static(io) & 1 != 0
    }

    fn sprites_on(&self) -> bool {
        self.get_gpu_control_register() & (1 << 1) != 0
    }

    fn large_sprites(&self) -> bool {
        self.get_gpu_control_register() & (1 << 2) != 0
    }

    fn bg_tile_map(&self) -> u8 {
        (self.get_gpu_control_register() & (1 << 3) != 0) as u8
    }

    fn bg_tile_set(&self) -> u8 {
        (self.get_gpu_control_register() & (1 << 4) != 0) as u8
    }

    fn window_on(&self) -> bool {
        self.get_gpu_control_register() & (1 << 5) != 0
    }

    fn window_tile_map(&self) -> u8 {
        (self.get_gpu_control_register() & (1 << 6) != 0) as u8
    }

    fn display_on(&self) -> bool {
        self.get_gpu_control_register() & (1 << 7) != 0
    }
}

const SCANLINE_OAM_TIME: u32 = 80;
const SCANLINE_VRAM_TIME: u32 = 172;
const HORIZONTAL_BLANK_TIME: u32 = 204;
const VERTICAL_BLANK_TIME: u32 = 4560;
const DIM_X: usize = 160;
const DIM_Y: u8 = 144;
const OFFSET_TILE_MAP_0: u16 = 0x1800;
const OFFSET_TILE_MAP_1: u16 = 0x1C00;
pub const CLOCK_TICKS_PER_FRAME: u32 = 70224;

enum Mode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    ScanlineOam = 2,
    ScanlineVram = 3,
}
