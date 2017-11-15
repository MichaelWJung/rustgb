use display::Display;
use memory::{Memory, BlockMemory};

pub struct Gpu<'a> {
    mode: Mode,
    mode_clock: u32,
    line: u8,
    vram: BlockMemory,
    sprites: BlockMemory,
    display: Display<'a>,
}

impl<'a> Gpu<'a> {
    pub fn new(display: Display) -> Gpu {
        Gpu {
            mode: Mode::HorizontalBlank,
            mode_clock: 0,
            line: 0,
            vram: BlockMemory::new(0x2000),
            sprites: BlockMemory::new(0xA0),
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
                    let new_line = self.line + 1;
                    self.set_current_line(new_line);
                    if self.line >= DIM_Y - 1 {
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
                    self.set_current_line(0);
                }
            }
        }
    }

    fn get_color(vram: &BlockMemory, tile: u16, x: u16, y: u16) -> u8 {
        let bit = 1 << (7 - x);
        let mut offset = tile * 0x10;
        offset += y as u16 * 0x2;
        let low_bit = (vram.read_byte(offset) & bit) != 0;
        let high_bit = (vram.read_byte(offset + 1) & bit) != 0;
        low_bit as u8 + high_bit as u8 * 2
    }

    fn render_scanline(&mut self) {
        let mut map_offset = if Self::bgmap(&self.sprites) {
            OFFSET_TILE_MAP_1
        } else {
            OFFSET_TILE_MAP_0
        };
        map_offset += ((self.line as u16 + self.scy()) & 0xFF) >> 3;
        let mut line_offset = self.scx() >> 3;
        let y = (self.line as u16 + self.scy()) & 0x7;
        let mut x = self.scx() & 0x7;
        let mut tile = self.vram.read_byte(map_offset + line_offset) as u16;
        if Self::bgmap(&self.sprites) && tile < 128 {
            tile += 256;
        }

        let display_line = self.display.get_line_mut(self.line);
        for i in 0..DIM_X {
            let color = Self::get_color(&self.vram, tile, x, y);
            display_line[i] = color;
            x = x % 8;
            if x == 0 {
                line_offset = (line_offset + 1) & 0x1F;
                tile = self.vram.read_byte(map_offset + line_offset) as u16;
                if Self::bgmap(&self.sprites) && tile < 128 {
                    tile += 256;
                }
            }
        }
    }

    fn render_screen(&mut self) {
        self.display.redraw();
    }

    fn bgmap(sprites: &BlockMemory) -> bool {
        false
    }

    fn scx(&self) -> u16 {
        self.sprites.read_byte(0x143) as u16
    }

    fn scy(&self) -> u16 {
        self.sprites.read_byte(0x142) as u16
    }

    fn set_current_line(&mut self, line: u8) {
        self.line = line;
        self.sprites.write_byte(0x144, line);
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

enum Mode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    ScanlineOam = 2,
    ScanlineVram = 3,
}
