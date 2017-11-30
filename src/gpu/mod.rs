mod constants;
mod palette;
mod sprite;
mod tile;

#[cfg(test)]
mod tests;

use display::{Display, COLS};
use memory::{Memory, BlockMemory};
use gpu::constants::*;
use gpu::palette::*;
use gpu::sprite::*;
use gpu::tile::*;
pub use gpu::constants::CLOCK_TICKS_PER_FRAME;
pub use gpu::tile::{TileMap, TileSet};

pub struct Gpu<D>
    where D: Display
{
    mode: Mode,
    mode_clock: u32,
    vram: BlockMemory,
    oam: BlockMemory,
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
    pub palettes: Palettes,

    pub vblank_interrupt_status: bool,
    pub state_interrupt_status: bool,

    pub state_interrupt_vblank: bool,
    pub state_interrupt_hblank: bool,
    pub state_interrupt_oam: bool,
    pub state_interrupt_lycly_coincidence: bool,
}

impl<D> Gpu<D>
    where D: Display
{
    pub fn new(display: D) -> Gpu<D> {
        let mut gpu = Gpu {
            mode: Mode::ScanlineOam,
            mode_clock: 0,
            vram: BlockMemory::new(0x2000),
            oam: BlockMemory::new(0x100),
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
            palettes: Palettes::new(),
            vblank_interrupt_status: false,
            state_interrupt_status: false,
            state_interrupt_vblank: false,
            state_interrupt_hblank: false,
            state_interrupt_oam: false,
            state_interrupt_lycly_coincidence: false,
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
                        self.vblank_interrupt_status = true;
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

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match mode {
            Mode::HorizontalBlank => {
                if self.state_interrupt_hblank {
                    self.fire_lcdc_interrupt();
                }
            }
            Mode::VerticalBlank => {
                if self.state_interrupt_vblank {
                    self.fire_lcdc_interrupt();
                }
            }
            Mode::ScanlineOam => {
                if self.state_interrupt_oam {
                    self.fire_lcdc_interrupt();
                }
            }
            _ => (),
        }
    }

    fn fire_lcdc_interrupt(&mut self) {
        self.state_interrupt_status = true;
    }

    fn render_scanline(&mut self) {
        if !self.display_on { return; }
        let display_line_number = self.get_current_line();
        let bg_pixels = self.render_bg_line(display_line_number);
        let sprite_pixels = self.render_sprites(display_line_number);
        let pixels = self.combine_pixels(bg_pixels, sprite_pixels);
        self.display.set_line(display_line_number, &pixels);
    }

    fn render_bg_line(&self, display_line_number: u8) -> [u8; COLS] {
        let mut pixels = [0; COLS];
        if !self.bg_on { return pixels; }

        let tile_set = self.bg_tile_set;
        let x = self.scx;
        let y = display_line_number as u16 + self.scy as u16;
        let mut tile_iter = self.bg_tile_map.get_tile_iter(x, y as u8, &self.vram);

        for i in 0..DIM_X {
            let tile = Tile::new(tile_iter.tile_number, tile_set);
            let color = tile.get_color(
                tile_iter.x as u8 % 8,
                tile_iter.y as u8 % 8,
                &self.vram
            );
            pixels[i] = color;
            tile_iter.next();
        }
        pixels
    }

    fn render_sprites(&self, display_line_number: u8) -> [SpritePixel; COLS] {
        let mut pixels = [SpritePixel { pixel: None, palette: None, priority: false }; COLS];
        if !self.sprites_on { return pixels; }
        let y = display_line_number as u16 + 16;
        let x = 8;
        let sprites = get_sprite_attributes_from_oam(&self.oam);
        for sprite in sprites.iter().rev() {
            let y_in_tile = y as i16 - sprite.y_position as i16;
            if y_in_tile < 0 || y_in_tile >= 8 { continue; }
            for i in 0..DIM_X {
                let x = i as u16 + x;
                let x_in_tile = x as i16 - sprite.x_position as i16;
                if x_in_tile < 0 || x_in_tile >= 8 { continue; }
                let tile = sprite.get_tile();
                let color = tile.get_color(
                    x_in_tile as u8,
                    y_in_tile as u8,
                    &self.vram,
                );
                let pixel = SpritePixel {
                    pixel: if color != 0 { Some(color) } else { None },
                    palette: Some(sprite.palette),
                    priority: sprite.priority
                };
                pixels[i] = pixel;
            }
        }
        pixels
    }

    fn combine_pixels(&self, bg_pixels: [u8; COLS], sprite_pixels: [SpritePixel; COLS]) -> [u8; COLS] {
        let mut combined = [0; COLS];
        for (i, (bg, sprite)) in bg_pixels
            .iter()
            .zip(sprite_pixels.iter())
            .enumerate()
        {
            combined[i] = if *bg != 0 && !sprite.priority {
                apply_palette(*bg, Palette::BackgroundPalette, &self.palettes)
            } else if let Some(color) = sprite.pixel {
                apply_palette(color, sprite.palette.unwrap(), &self.palettes)
            } else {
                apply_palette(*bg, Palette::BackgroundPalette, &self.palettes)
            }
        }
        combined
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
        if self.state_interrupt_lycly_coincidence && self.lyc == self.current_line {
            self.fire_lcdc_interrupt();
        }
    }
}

#[derive(Copy, Clone)]
enum Mode {
    HorizontalBlank = 0,
    VerticalBlank = 1,
    ScanlineOam = 2,
    ScanlineVram = 3,
}
