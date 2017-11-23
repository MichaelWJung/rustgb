use display::{Display, COLS};
use memory::{Memory, BlockMemory};
use std::cell::RefCell;

pub struct Gpu<'a, D>
    where D: Display + 'a
{
    mode: Mode,
    mode_clock: u32,
    vram: BlockMemory,
    oam: BlockMemory,
    io: &'a RefCell<BlockMemory>,
    display: D,
    pub bg_on: bool,
    pub sprites_on: bool,
    pub large_sprites: bool,
    pub bg_tile_map: TileMap,
    pub bg_tile_set: TileSet,
    pub window_on: bool,
    pub window_tile_map: TileMap,
    display_on: bool,
    pub scx: u8,
    pub scy: u8,
    current_line: u8,
    pub lyc: u8,
}

impl<'a, D> Gpu<'a, D>
    where D: Display + 'a
{
    pub fn new(display: D, io: &'a RefCell<BlockMemory>) -> Gpu<'a, D> {
        let mut gpu = Gpu {
            mode: Mode::ScanlineOam,
            mode_clock: 0,
            vram: BlockMemory::new(0x2000),
            oam: BlockMemory::new(0x100),
            io,
            display,
            bg_on: false,
            sprites_on: false,
            large_sprites: false,
            bg_tile_map: TileMap::Map0,
            bg_tile_set: TileSet::Set0,
            window_on: false,
            window_tile_map: TileMap::Map0,
            display_on: false,
            scx: 0,
            scy: 0,
            current_line: 0,
            lyc: 0,
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

    pub fn get_oam(&self) -> &Memory {
        &self.oam
    }

    pub fn get_oam_mut(&mut self) -> &mut Memory {
        &mut self.oam
    }

    pub fn get_mode(&self) -> u8 {
        match self.mode {
            Mode::HorizontalBlank => 0,
            Mode::VerticalBlank => 1,
            Mode::ScanlineOam => 2,
            Mode::ScanlineVram => 3,
        }
    }

    pub fn set_display_on(&mut self, on: bool) {
        self.display_on = on;
    }

    pub fn get_display_on(&self) -> bool {
        self.display_on
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::HorizontalBlank => {
                if InterruptMode::HorizontalBlank.is_set(&self.io.borrow()) {
                    self.fire_lcdc_interrupt();
                }
            }
            Mode::VerticalBlank => {
                if InterruptMode::VerticalBlank.is_set(&self.io.borrow()) {
                    self.fire_lcdc_interrupt();
                }
            }
            Mode::ScanlineOam => {
                if InterruptMode::ScanlineOam.is_set(&self.io.borrow()) {
                    self.fire_lcdc_interrupt();
                }
            }
            _ => (),
        }
    }

    fn fire_lcdc_interrupt(&mut self) {
        let interrupts_fired = self.io.borrow().read_byte(0x0F);
        self.io.borrow_mut().write_byte(0x0F, interrupts_fired | 0b0000_0010);
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
                    if self.get_current_line() == DIM_Y - 1 {
                        self.set_mode(Mode::VerticalBlank);
                        self.render_screen();
                        let interrupts_fired = self.io.borrow().read_byte(0x0F);
                        self.io.borrow_mut().write_byte(0x0F, interrupts_fired | 0b0000_0001);
                    } else {
                        self.set_mode(Mode::ScanlineOam);
                    }
                    self.increment_current_line();
                }
            }
            Mode::VerticalBlank => {
                if self.mode_clock >= VERTICAL_BLANK_TIME / 10 {
                    self.mode_clock %= VERTICAL_BLANK_TIME / 10;
                    self.increment_current_line();
                    if self.get_current_line() > 153 {
                        self.set_mode(Mode::ScanlineOam);
                        self.reset_current_line();
                    }
                }
            }
        }
    }

    fn render_scanline(&mut self) {
        if !self.display_on { return; }
        let display_line_number = self.get_current_line();
        let mut display_line_memory = [0; COLS];
        self.render_bg_line(display_line_number, &mut display_line_memory);
        self.render_sprites(display_line_number, &mut display_line_memory);
        self.display.set_line(display_line_number, &display_line_memory);
    }

    fn render_bg_line(&self, display_line_number: u8, display_line_memory: &mut [u8]) {
        if !self.bg_on { return; }
        let tile_set = self.bg_tile_set;
        let x = self.scx;
        let y = display_line_number as u16 + self.scy as u16;
        let mut tile_iter = self.bg_tile_map.get_tile_iter(x, y as u8, &self.vram);

        for i in 0..DIM_X {
            let tile = Tile::new(tile_iter.tile_number, tile_set, Palette::BackgroundPalette);
            let color = tile.get_color(
                tile_iter.x as u8 % 8,
                tile_iter.y as u8 % 8,
                &self.vram,
                &self.io.borrow()
            );
            display_line_memory[i] = color;
            tile_iter.next();
        }
    }

    fn render_sprites(&self, display_line_number: u8, display_line_memory: &mut [u8]) {
        if !self.sprites_on { return; }
        let y = display_line_number as u16 + self.scy as u16 + 16;
        let x = self.scx as u16 + 8;
        let sprites = get_sprite_attributes_from_oam(&self.oam);
        for sprite in sprites.iter().rev() {
            let y_in_tile = y as i16 - sprite.y_position as i16;
            if y_in_tile < 0 || y_in_tile >= 8 { continue; }
            for i in 0..DIM_X {
                let x = i as u16 + x;
                let x_in_tile = x as i16 - sprite.x_position as i16;
                if x_in_tile < 0 || x_in_tile >= 8 { continue; }
                let tile = sprite.get_tile(&self.vram);
                let color = tile.get_color(
                    x_in_tile as u8,
                    y_in_tile as u8,
                    &self.vram,
                    &self.io.borrow()
                );
                display_line_memory[i] = color;
            }
        }
    }

    fn render_screen(&mut self) {
        self.display.redraw();
    }

    fn increment_current_line(&mut self) -> u8 {
        self.current_line += 1;
        self.check_fire_coincidence_interrupt();
        self.current_line
    }

    pub fn get_current_line(&self) -> u8 {
        self.current_line
    }

    fn reset_current_line(&mut self) {
        self.current_line = 0;
        self.check_fire_coincidence_interrupt();
    }

    fn check_fire_coincidence_interrupt(&mut self) {
        if self.lyc == self.current_line {

        }
        //let mut state = self.io.borrow().read_byte(0x41);
        //state &= 0b1111_1011;
        //state |= (value as u8) << 2;
        //self.io.borrow_mut().write_byte(0x41, state);
        //if InterruptMode::LycLyConcidence.is_set(&self.io.borrow()) {
        //    self.fire_lcdc_interrupt();
        //}
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

enum InterruptMode {
    LycLyConcidence,
    ScanlineOam,
    VerticalBlank,
    HorizontalBlank,
}

impl InterruptMode {
    fn is_set(&self, io: &BlockMemory) -> bool {
        let stat = io.read_byte(0x41);
        match *self {
            InterruptMode::LycLyConcidence => stat & 0b0100_0000 != 0,
            InterruptMode::ScanlineOam => stat & 0b0010_0000 != 0,
            InterruptMode::VerticalBlank => stat & 0b0001_0000 != 0,
            InterruptMode::HorizontalBlank => stat & 0b0000_1000 != 0,
        }
    }
}

#[derive(Copy, Clone)]
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
pub enum TileSet {
    Set0,
    Set1,
}

impl TileSet {
    pub fn from_bool(b: bool) -> TileSet {
        if b { TileSet::Set1 } else { TileSet::Set0 }
    }

    pub fn to_bool(&self) -> bool {
        match *self {
            TileSet::Set0 => false,
            TileSet::Set1 => true,
        }
    }
}

#[derive(Copy, Clone)]
struct Tile {
    tile_num: u8,
    tile_set: TileSet,
    x_flip: bool,
    y_flip: bool,
    large_tile: bool,
    palette: Palette,
}

impl Tile {
    fn new(tile_num: u8, tile_set: TileSet, palette: Palette) -> Tile {
        Tile {
            tile_num,
            tile_set,
            x_flip: false,
            y_flip: false,
            large_tile: false,
            palette,
        }
    }

    fn get_color<M: Memory>(&self, x: u8, y: u8, vram: &M, io: &M) -> u8 {
        let line = self.get_line(y, vram);
        let color = line[x as usize];
        apply_palette(color, self.palette, io)
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

pub enum TileMap {
    Map0,
    Map1,
}

impl TileMap {
    pub fn from_bool(b: bool) -> TileMap {
        if b { TileMap::Map1 } else { TileMap::Map0 }
    }

    pub fn to_bool(&self) -> bool {
        match *self {
            TileMap::Map0 => false,
            TileMap::Map1 => true,
        }
    }

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

    fn get_tile<M: Memory>(&self, vram: &M) -> Tile {
        Tile {
            tile_num: self.tile_num,
            tile_set: TileSet::Set1,
            x_flip: self.x_flip,
            y_flip: self.y_flip,
            large_tile: false,
            palette: self.palette,
        }
    }
}

fn get_sprite_attributes_from_oam(oam: &BlockMemory) -> Vec<SpriteAttribute> {
    let mut attributes = Vec::new();
    attributes.reserve_exact(NUM_SPRITES as usize);
    for i in 0..NUM_SPRITES {
        let from = i * 0x4;
        attributes.push(SpriteAttribute::new(&oam.read_4_bytes(from)));
    }
    attributes
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

        fn set_line(&mut self, line: u8, pixels: &[u8; COLS]) {
            let line = line as usize;
            self.pixels[(COLS * line)..(COLS * (line + 1))].copy_from_slice(pixels);
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
                gpu.set_display_on(true);
                gpu.bg_tile_set = TileSet::Set0;
                gpu.bg_tile_map = TileMap::Map0;
                gpu.bg_on = true;
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
                gpu.set_display_on(true);
                gpu.bg_tile_set = TileSet::Set0;
                gpu.bg_tile_map = TileMap::Map1;
                gpu.bg_on = true;
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
                gpu.set_display_on(true);
                gpu.bg_tile_set = TileSet::Set1;
                gpu.bg_tile_map = TileMap::Map0;
                gpu.bg_on = true;
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
