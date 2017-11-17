use display::Display;
use memory::{Memory, BlockMemory};
use std::cell::RefCell;
use std::ops::Deref;

pub struct Gpu<'a, D>
    where D: Display + 'a
{
    mode: Mode,
    mode_clock: u32,
    vram: BlockMemory,
    sprites: BlockMemory,
    io: &'a RefCell<BlockMemory>,
    display: D,
}

impl<'a, D> Gpu<'a, D>
    where D: Display + 'a
{
    pub fn new(display: D, io: &'a RefCell<BlockMemory>) -> Gpu<'a, D> {
        Gpu {
            mode: Mode::HorizontalBlank,
            mode_clock: 0,
            vram: BlockMemory::new(0x2000),
            sprites: BlockMemory::new(0x100),
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
                    if new_line >= DIM_Y {
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
        let palette = io.read_byte(OFFSET_PALETTE);
        palette >> (color * 2) & 3
    }

    fn render_scanline(&mut self) {
        let mut map_offset = if Self::bg_tile_map(&self.io.borrow()) {
            OFFSET_TILE_MAP_1
        } else {
            OFFSET_TILE_MAP_0
        };
        map_offset += ((self.get_current_line() as u16 + self.scy()) & 0xFF) >> 3;
        let mut line_offset = self.scx() >> 3;
        let y = (self.get_current_line() as u16 + self.scy()) & 0x7;
        let mut x = self.scx() & 0x7;
        let mut tile = self.vram.read_byte(map_offset + line_offset) as u16;
        if !Self::bg_tile_set(&self.io.borrow()) && tile < 128 {
            tile += 256;
        }

        let current_line = self.get_current_line();
        let display_line = self.display.get_line_mut(current_line);
        for i in 0..DIM_X {
            let color = Self::get_color(&self.vram, &self.io.borrow(), tile, x, y);
            display_line[i] = color;
            x = (x + 1) % 8;
            if x == 0 {
                line_offset = (line_offset + 1) & 0x1F;
                tile = self.vram.read_byte(map_offset + line_offset) as u16;
                if !Self::bg_tile_set(&self.io.borrow()) && tile < 128 {
                    tile += 256;
                }
            }
        }
    }

    fn render_screen(&mut self) {
        self.display.redraw();
    }

    fn scx(&self) -> u16 {
        //self.io.borrow().read_byte(0x43) as u16
        0
    }

    fn scy(&self) -> u16 {
        //self.io.borrow().read_byte(0x42) as u16
        0
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
        io.read_byte(OFFSET_LCD_CONTROL_REGISTER)
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

    fn bg_tile_map(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register_static(io) & (1 << 3) != 0
    }

    fn bg_tile_set(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register_static(io) & (1 << 4) != 0
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
const OFFSET_TILE_SET_1: u16 = 0x0000;
// Offset to the 0th tile. There are negative tile numbers in tile set 1
const OFFSET_TILE_SET_0: u16 = 0x1000;
const OFFSET_TILE_MAP_0: u16 = 0x1800;
const OFFSET_TILE_MAP_1: u16 = 0x1C00;
const OFFSET_LCD_CONTROL_REGISTER: u16 = 0x0040;
const OFFSET_PALETTE: u16 = 0x0047;
pub const CLOCK_TICKS_PER_FRAME: u32 = 70224;

enum Mode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    ScanlineOam = 2,
    ScanlineVram = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use display::{Display, PIXELS, COLS};
    struct MockDisplay<'a> {
        pixels: &'a mut [u8; PIXELS],
    }

    impl<'a> MockDisplay<'a> {
        fn new(pixels: &mut [u8; PIXELS]) -> MockDisplay {
            MockDisplay { pixels }
        }
    }

    impl<'a> Display for MockDisplay<'a> {
        fn redraw(&mut self) {}

        fn clear(&mut self) {
            *self.pixels = [0; PIXELS];
        }

        fn get_line_mut(&mut self, line: u8) -> &mut[u8] {
            let line = line as usize;
            &mut self.pixels[(COLS * line)..(COLS * (line + 1))]
        }
    }

    #[test]
    fn test_simple_background_tile_map_0_set_0() {
        let mut pixels = [0; PIXELS];
        {
            let display = MockDisplay::new(&mut pixels);
            let io = RefCell::new(BlockMemory::new(0x80));
            let mut gpu = Gpu::new(display, &io);
            {
                let mut io = io.borrow_mut();
                io.write_byte(OFFSET_PALETTE, 0b11100100);
                io.write_byte(OFFSET_LCD_CONTROL_REGISTER, 0b10000000);
                let vram = gpu.get_vram_mut();
                vram.write_byte(OFFSET_TILE_SET_0 + 0x00, 0b10101010);
                vram.write_byte(OFFSET_TILE_SET_0 + 0x01, 0b10101010);
                vram.write_byte(OFFSET_TILE_SET_0 + 0x10, 0b00000000);
                vram.write_byte(OFFSET_TILE_SET_0 + 0x11, 0b11000110);
                vram.write_byte(OFFSET_TILE_SET_0 - 0x10, 0b00111100);
                vram.write_byte(OFFSET_TILE_SET_0 - 0x0F, 0b00011000);
                vram.write_byte(OFFSET_TILE_MAP_0, 0);
                vram.write_byte(OFFSET_TILE_MAP_0 + 1, 1);
                vram.write_byte(OFFSET_TILE_MAP_0 + 2, -1i8 as u8);
            }
            gpu.render_scanline();
        }
        assert_eq!(3, pixels[0x0]);
        assert_eq!(0, pixels[0x1]);
        assert_eq!(3, pixels[0x2]);
        assert_eq!(0, pixels[0x3]);
        assert_eq!(3, pixels[0x4]);
        assert_eq!(0, pixels[0x5]);
        assert_eq!(3, pixels[0x6]);
        assert_eq!(0, pixels[0x7]);

        assert_eq!(2, pixels[0x8]);
        assert_eq!(2, pixels[0x9]);
        assert_eq!(0, pixels[0xA]);
        assert_eq!(0, pixels[0xB]);
        assert_eq!(0, pixels[0xC]);
        assert_eq!(2, pixels[0xD]);
        assert_eq!(2, pixels[0xE]);
        assert_eq!(0, pixels[0xF]);

        assert_eq!(0, pixels[0x10]);
        assert_eq!(0, pixels[0x11]);
        assert_eq!(1, pixels[0x12]);
        assert_eq!(3, pixels[0x13]);
        assert_eq!(3, pixels[0x14]);
        assert_eq!(1, pixels[0x15]);
        assert_eq!(0, pixels[0x16]);
        assert_eq!(0, pixels[0x17]);
    }

    #[test]
    fn test_simple_background_tile_map_1_set_0() {
        let mut pixels = [0; PIXELS];
        {
            let display = MockDisplay::new(&mut pixels);
            let io = RefCell::new(BlockMemory::new(0x80));
            let mut gpu = Gpu::new(display, &io);
            {
                let mut io = io.borrow_mut();
                io.write_byte(OFFSET_PALETTE, 0b11100100);
                io.write_byte(OFFSET_LCD_CONTROL_REGISTER, 0b10001000);
                let vram = gpu.get_vram_mut();
                vram.write_byte(OFFSET_TILE_SET_0 + 0x00, 0b10101010);
                vram.write_byte(OFFSET_TILE_SET_0 + 0x01, 0b10101010);
                vram.write_byte(OFFSET_TILE_SET_0 + 0x10, 0b00000000);
                vram.write_byte(OFFSET_TILE_SET_0 + 0x11, 0b11000110);
                vram.write_byte(OFFSET_TILE_MAP_1, 0);
                vram.write_byte(OFFSET_TILE_MAP_1 + 1, 1);
            }
            gpu.render_scanline();
        }
        assert_eq!(3, pixels[0x0]);
        assert_eq!(0, pixels[0x1]);
        assert_eq!(3, pixels[0x2]);
        assert_eq!(0, pixels[0x3]);
        assert_eq!(3, pixels[0x4]);
        assert_eq!(0, pixels[0x5]);
        assert_eq!(3, pixels[0x6]);
        assert_eq!(0, pixels[0x7]);

        assert_eq!(2, pixels[0x8]);
        assert_eq!(2, pixels[0x9]);
        assert_eq!(0, pixels[0xA]);
        assert_eq!(0, pixels[0xB]);
        assert_eq!(0, pixels[0xC]);
        assert_eq!(2, pixels[0xD]);
        assert_eq!(2, pixels[0xE]);
        assert_eq!(0, pixels[0xF]);
    }

    #[test]
    fn test_simple_background_tile_map_0_set_1() {
        let mut pixels = [0; PIXELS];
        {
            let display = MockDisplay::new(&mut pixels);
            let io = RefCell::new(BlockMemory::new(0x80));
            let mut gpu = Gpu::new(display, &io);
            {
                let mut io = io.borrow_mut();
                io.write_byte(OFFSET_PALETTE, 0b11100100);
                io.write_byte(OFFSET_LCD_CONTROL_REGISTER, 0b10010000);
                let vram = gpu.get_vram_mut();
                vram.write_byte(OFFSET_TILE_SET_1 + 0x00, 0b10101010);
                vram.write_byte(OFFSET_TILE_SET_1 + 0x01, 0b10101010);
                vram.write_byte(OFFSET_TILE_SET_1 + 0x10, 0b00000000);
                vram.write_byte(OFFSET_TILE_SET_1 + 0x11, 0b11000110);
                vram.write_byte(OFFSET_TILE_MAP_0, 0);
                vram.write_byte(OFFSET_TILE_MAP_0 + 1, 1);
            }
            gpu.render_scanline();
        }
        assert_eq!(3, pixels[0x0]);
        assert_eq!(0, pixels[0x1]);
        assert_eq!(3, pixels[0x2]);
        assert_eq!(0, pixels[0x3]);
        assert_eq!(3, pixels[0x4]);
        assert_eq!(0, pixels[0x5]);
        assert_eq!(3, pixels[0x6]);
        assert_eq!(0, pixels[0x7]);

        assert_eq!(2, pixels[0x8]);
        assert_eq!(2, pixels[0x9]);
        assert_eq!(0, pixels[0xA]);
        assert_eq!(0, pixels[0xB]);
        assert_eq!(0, pixels[0xC]);
        assert_eq!(2, pixels[0xD]);
        assert_eq!(2, pixels[0xE]);
        assert_eq!(0, pixels[0xF]);
    }
}
