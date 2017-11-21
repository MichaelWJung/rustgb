use display::Display;
use memory::{Memory, BlockMemory};
use std::cell::RefCell;

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
        let mut gpu = Gpu {
            mode: Mode::ScanlineOam,
            mode_clock: 0,
            vram: BlockMemory::new(0x2000),
            sprites: BlockMemory::new(0x100),
            io,
            display,
        };
        gpu.set_mode(Mode::ScanlineOam);
        gpu
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

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        let mode_bits = match mode {
            Mode::HorizontalBlank => 0,
            Mode::VerticalBlank => 1,
            Mode::ScanlineOam => 2,
            Mode::ScanlineVram => 3,
        };
        let mut state = self.io.borrow().read_byte(0x41);
        state &= 0b11111100;
        state |= mode_bits;
        self.io.borrow_mut().write_byte(0x41, state);
    }

    pub fn step(&mut self, cycles: u8) {
        let cycles = cycles as u32;
        self.mode_clock += cycles;
        match self.mode {
            Mode::ScanlineOam => {
                if self.mode_clock >= SCANLINE_OAM_TIME {
                    self.mode_clock %= SCANLINE_OAM_TIME;
                    self.set_mode(Mode::ScanlineVram);
                }
            }
            Mode::ScanlineVram => {
                if self.mode_clock >= SCANLINE_VRAM_TIME {
                    self.mode_clock %= SCANLINE_VRAM_TIME;
                    self.set_mode(Mode::HorizontalBlank);
                    self.render_scanline();
                }
            }
            Mode::HorizontalBlank => {
                if self.mode_clock >= HORIZONTAL_BLANK_TIME {
                    self.mode_clock %= HORIZONTAL_BLANK_TIME;
                    let new_line = self.increment_current_line();
                    if new_line >= DIM_Y {
                        self.set_mode(Mode::VerticalBlank);
                        self.render_screen();
                    } else {
                        self.set_mode(Mode::ScanlineOam);
                    }
                }
            }
            Mode::VerticalBlank => {
                if self.mode_clock >= VERTICAL_BLANK_TIME {
                    self.mode_clock %= VERTICAL_BLANK_TIME;
                    self.set_mode(Mode::ScanlineOam);
                    self.reset_current_line();
                }
            }
        }
    }

    fn get_color(vram: &BlockMemory, io: &BlockMemory, tile: Tile, x: u16, y: u16) -> u8 {
        let line = tile.get_line(y as u8, vram);
        let color = line[x as usize];
        apply_palette(color, Palette::BackgroundPalette, io)
    }

    fn render_scanline(&mut self) {
        let tile_map = Self::bg_tile_map(&self.io.borrow());
        let tile_set = Self::bg_tile_set(&self.io.borrow());
        let display_line_number = self.get_current_line();
        let x = self.scx();
        let y = display_line_number as u16 + self.scy();
        let mut tile_iter = tile_map.get_tile_iter(x as u8, y as u8, &self.vram);

        let display_line_memory = self.display.get_line_mut(display_line_number);
        for i in 0..DIM_X {
            let tile = Tile::new(tile_iter.tile_number, tile_set);
            let color = Self::get_color(
                &self.vram,
                &self.io.borrow(),
                tile,
                tile_iter.x as u16 % 8,
                tile_iter.y as u16 % 8
            );
            display_line_memory[i] = color;
            tile_iter.next();
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

    fn get_gpu_control_register(io: &BlockMemory) -> u8 {
        io.read_byte(OFFSET_LCD_CONTROL_REGISTER)
    }

    fn bg_on(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register(io) & 1 != 0
    }

    fn sprites_on(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register(io) & (1 << 1) != 0
    }

    fn _large_sprites(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register(io) & (1 << 2) != 0
    }

    fn bg_tile_map(io: &BlockMemory) -> TileMap {
        if Self::get_gpu_control_register(io) & (1 << 3) != 0 {
            TileMap::Map1
        } else {
            TileMap::Map0
        }
    }

    fn bg_tile_set(io: &BlockMemory) -> TileSet {
        if Self::get_gpu_control_register(io) & (1 << 4) != 0 {
            TileSet::Set1
        } else {
            TileSet::Set0
        }
    }

    fn _window_on(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register(io) & (1 << 5) != 0
    }

    fn _window_tile_map(io: &BlockMemory) -> u8 {
        (Self::get_gpu_control_register(io) & (1 << 6) != 0) as u8
    }

    fn _display_on(io: &BlockMemory) -> bool {
        Self::get_gpu_control_register(io) & (1 << 7) != 0
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
const TILE_SIZE_IN_BYTES: u16 = 0x10;
const OFFSET_LCD_CONTROL_REGISTER: u16 = 0x0040;
const OFFSET_BACKGROUND_PALETTE: u16 = 0x0047;
const OFFSET_OBJECT0_PALETTE: u16 = 0x0048;
const OFFSET_OBJECT1_PALETTE: u16 = 0x0048;
pub const CLOCK_TICKS_PER_FRAME: u32 = 70224;
const NUM_SPRITES: u16 = 40;

#[derive(Copy, Clone)]
enum Mode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    ScanlineOam = 2,
    ScanlineVram = 3,
}

enum Palette {
    BackgroundPalette,
    ObjectPalette0,
    ObjectPalette1,
}

fn apply_palette<M: Memory>(color: u8, palette: Palette, io: &M) -> u8 {
    let address = match palette {
        Palette::BackgroundPalette => OFFSET_BACKGROUND_PALETTE,
        Palette::ObjectPalette0 => OFFSET_OBJECT0_PALETTE,
        Palette::ObjectPalette1 => OFFSET_OBJECT1_PALETTE,
    };
    let palette = io.read_byte(address);
    palette >> (color * 2) & 3
}

#[derive(Copy, Clone)]
enum TileSet {
    Set0,
    Set1,
}

#[derive(Copy, Clone)]
struct Tile {
    tile_num: u8,
    tile_set: TileSet,
}

impl Tile {
    fn new(tile_num: u8, tile_set: TileSet) -> Tile {
        Tile { tile_num, tile_set }
    }

    fn get_line<M: Memory>(&self, line_num: u8, vram: &M) -> [u8; 8] {
        let address = self.get_line_address(line_num);
        let low_bits = vram.read_byte(address);
        let high_bits = vram.read_byte(address + 1);
        let mut line = [0; 8];
        for i in 0..8 {
            if low_bits & (0x80 >> i) != 0 {
                line[i] += 1;
            }
            if high_bits & (0x80 >> i) != 0 {
                line[i] += 2;
            }
        }
        line
    }

    fn get_line_address(&self, line_num: u8) -> u16 {
        let base_offset;
        let tile_offset;
        match self.tile_set {
            TileSet::Set0 => {
                base_offset = OFFSET_TILE_SET_0;
                tile_offset = self.tile_num as i8 as i32 * TILE_SIZE_IN_BYTES as i32;
            }
            TileSet::Set1 => {
                base_offset = OFFSET_TILE_SET_1;
                tile_offset = self.tile_num as i32 * TILE_SIZE_IN_BYTES as i32;
            }
        };
        (base_offset as i32 + tile_offset) as u16 + line_num as u16 * 0x2
    }
}

enum TileMap {
    Map0,
    Map1,
}

impl TileMap {
    fn get_tile_iter<'a, 'b: 'a, M: Memory>(&'a self, x: u8, y: u8, vram: &'b M) -> TileIterator<M> {
        let base_offset = match *self {
            TileMap::Map0 => OFFSET_TILE_MAP_0,
            TileMap::Map1 => OFFSET_TILE_MAP_1,
        };
        let row = y / 8;
        let col = x / 8;
        let tile_offset = row as u16 * 32 + col as u16;
        let tile_number = vram.read_byte(base_offset + tile_offset);
        TileIterator { bg_x: x, x: x % 8, y, tile_number, tile_map: self, vram }
    }
}

struct TileIterator<'a, M>
    where M: Memory + 'a
{
    bg_x: u8,
    x: u8,
    y: u8,
    tile_number: u8,
    tile_map: &'a TileMap,
    vram: &'a M
}

impl<'a, M> TileIterator<'a, M>
    where M: Memory
{
    fn next(&mut self) {
        self.x = (self.x + 1) % 8;
        self.bg_x = self.bg_x.wrapping_add(1);
        if self.x == 0 {
            *self = self.tile_map.get_tile_iter(self.bg_x, self.y, self.vram);
        }
    }
}

struct SpriteAttribute {
    x_position: u8,
    y_position: u8,
    tile_num: u8,
    priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette: Palette,
}

impl SpriteAttribute {
    fn new(memory: &[u8]) -> SpriteAttribute {
        let x_position = memory[1];
        let y_position = memory[0];
        let tile_num = memory[2];
        let flags = memory[3];
        let priority = flags & 0x80 == 0;
        let x_flip = flags & 0x20 != 0;
        let y_flip = flags & 0x40 != 0;
        let palette = if flags & 0x10 != 0 {
            Palette::ObjectPalette1
        } else {
            Palette::ObjectPalette0
        };
        SpriteAttribute {
            x_position,
            y_position,
            tile_num,
            priority,
            x_flip,
            y_flip,
            palette,
        }
    }

    fn attributes_from_oam(oam: &BlockMemory) -> Vec<SpriteAttribute> {
        let mut attributes = Vec::new();
        attributes.reserve_exact(NUM_SPRITES as usize);
        for i in 0..NUM_SPRITES {
            let from = i * 0x4;
            attributes.push(Self::new(&oam.read_4_bytes(from)));
        }
        attributes
    }
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
                io.write_byte(OFFSET_BACKGROUND_PALETTE, 0b11100100);
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
                io.write_byte(OFFSET_BACKGROUND_PALETTE, 0b11100100);
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
                io.write_byte(OFFSET_BACKGROUND_PALETTE, 0b11100100);
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
