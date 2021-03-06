mod constants;
mod palette;
mod sprite;
mod tile;
mod vram;

#[cfg(test)]
mod tests;

use display::{Display, COLS};
use memory::{Memory, BlockMemory};
use self::constants::*;
use self::palette::*;
use self::sprite::*;
use self::tile::{Tile, TileIterator};
use self::vram::Vram;
use std::cmp;
pub use self::constants::CLOCK_TICKS_PER_FRAME;

pub struct GpuState {
    mode: Mode,
    pub bg_on: bool,
    pub sprites_on: bool,
    pub large_sprites: bool,
    pub bg_tile_map: TileMap,
    pub bg_window_tile_set: TileSet,
    pub window_on: bool,
    pub window_tile_map: TileMap,
    display_on: bool,
    pub scx: u8,
    pub scy: u8,
    current_line: u8,
    pub lyc: u8,
    pub window_x: u8,
    pub window_y: u8,
    pub palettes: Palettes,

    pub vblank_interrupt_status: bool,
    pub state_interrupt_status: bool,

    pub state_interrupt_vblank: bool,
    pub state_interrupt_hblank: bool,
    pub state_interrupt_oam: bool,
    pub state_interrupt_lycly_coincidence: bool,
}

impl GpuState {
    pub fn set_display_on(&mut self, on: bool) {
        self.display_on = on;
    }

    pub fn get_display_on(&self) -> bool {
        self.display_on
    }

    pub fn get_mode(&self) -> u8 {
        match self.mode {
            Mode::HorizontalBlank => 0,
            Mode::VerticalBlank => 1,
            Mode::ScanlineOam => 2,
            Mode::ScanlineVram => 3,
        }
    }

    pub fn get_current_line(&self) -> u8 {
        self.current_line
    }
}

pub struct Gpu<D>
    where D: Display
{
    mode_clock: u32,
    vram: Vram,
    oam: BlockMemory,
    display: D,
    pub state: GpuState,
}

impl<D> Gpu<D>
    where D: Display
{
    pub fn new(display: D) -> Gpu<D> {
        let mut gpu = Gpu {
            mode_clock: 0,
            vram: Vram::new(),
            oam: BlockMemory::new(0x100),
            display,
            state: GpuState {
                mode: Mode::ScanlineOam,
                bg_on: false,
                sprites_on: false,
                large_sprites: false,
                bg_tile_map: TileMap::Map0,
                bg_window_tile_set: TileSet::Set0,
                window_on: false,
                window_tile_map: TileMap::Map0,
                display_on: false,
                scx: 0,
                scy: 0,
                window_x: 0,
                window_y: 0,
                current_line: 0,
                lyc: 0,
                palettes: Palettes::new(),
                vblank_interrupt_status: false,
                state_interrupt_status: false,
                state_interrupt_vblank: false,
                state_interrupt_hblank: false,
                state_interrupt_oam: false,
                state_interrupt_lycly_coincidence: false,
            },
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

    pub fn step(&mut self, cycles: u8) {
        let cycles = cycles as u32;
        self.mode_clock += cycles;
        match self.state.mode {
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
                    if self.state.get_current_line() == DIM_Y - 1 {
                        self.set_mode(Mode::VerticalBlank);
                        self.render_screen();
                        self.state.vblank_interrupt_status = true;
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
                    if self.state.get_current_line() > 153 {
                        self.set_mode(Mode::ScanlineOam);
                        self.reset_current_line();
                    }
                }
            }
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.state.mode = mode;
        match mode {
            Mode::HorizontalBlank => {
                if self.state.state_interrupt_hblank {
                    self.fire_lcdc_interrupt();
                }
            }
            Mode::VerticalBlank => {
                if self.state.state_interrupt_vblank {
                    self.fire_lcdc_interrupt();
                }
            }
            Mode::ScanlineOam => {
                if self.state.state_interrupt_oam {
                    self.fire_lcdc_interrupt();
                }
            }
            _ => (),
        }
    }

    fn fire_lcdc_interrupt(&mut self) {
        self.state.state_interrupt_status = true;
    }

    fn render_scanline(&mut self) {
        if !self.state.display_on { return; }
        let display_line_number = self.state.get_current_line();
        let bg_pixels = self.render_bg_line(display_line_number);
        let window_pixels = self.render_window_line(display_line_number);
        let sprite_pixels = self.render_sprites(display_line_number);
        let pixels = self.combine_pixels(bg_pixels, window_pixels, sprite_pixels);
        self.display.set_line(display_line_number, &pixels);
    }

    fn render_bg_line(&self, display_line_number: u8) -> [u8; COLS] {
        let mut pixels = [0; COLS];
        if !self.state.bg_on { return pixels; }

        let x = self.state.scx;
        let y = display_line_number.wrapping_add(self.state.scy);
        let mut tile_iter = TileIterator::new(x, y, self.state.bg_tile_map, &self.vram);
        for i in 0..DIM_X {
            pixels[i] = tile_iter.get_pixel_color(self.state.bg_window_tile_set);
            tile_iter.next();
        }
        pixels
    }

    fn render_window_line(&self, display_line_number: u8) -> [Option<u8>; COLS] {
        let mut pixels = [None; COLS];
        if !self.state.window_on { return pixels; }

        if display_line_number >= self.state.window_y {
            let start_x = cmp::max(0, self.state.window_x as i16 - 7) as usize;
            let window_y = display_line_number - self.state.window_y;
            let mut tile_iter = TileIterator::new(0, window_y, self.state.window_tile_map, &self.vram);
            for i in start_x..DIM_X {
                pixels[i] = Some(tile_iter.get_pixel_color(self.state.bg_window_tile_set));
                tile_iter.next();
            }
        }
        pixels
    }

    fn render_sprites(&self, display_line_number: u8) -> [SpritePixel; COLS] {
        let mut pixels = [SpritePixel { pixel: None, palette: None, priority: false }; COLS];
        if !self.state.sprites_on { return pixels; }
        let sprite_y_size = if self.state.large_sprites { 16 } else { 8 };
        let y = display_line_number as u16 + 16;
        let x = 8;
        let sprites = get_sprite_attributes_from_oam(&self.oam, self.state.large_sprites);
        let mut sprites: Vec<_> = sprites.iter().filter(|s| {
            let y_in_tile = y as i16 - s.get_y_pos() as i16;
            y_in_tile >= 0 && y_in_tile < sprite_y_size
        }).take(10).collect();
        sprites.sort_by(|a, b| { a.get_x_pos().cmp(&b.get_x_pos()) });
        for sprite in sprites {
            let y_in_tile = y as i16 - sprite.get_y_pos() as i16;
            for i in 0..DIM_X {
                if let Some(_) = pixels[i].pixel {
                    continue;
                }
                let x = i as u16 + x;
                let x_in_tile = x as i16 - sprite.get_x_pos() as i16;
                if x_in_tile < 0 || x_in_tile >= 8 { continue; }
                let tile = Tile::from_sprite(sprite);
                let color = tile.get_color(
                    x_in_tile as u8,
                    y_in_tile as u8,
                    &self.vram,
                );
                if color != 0 {
                    pixels[i] = SpritePixel {
                        pixel: Some(color),
                        palette: Some(sprite.get_palette()),
                        priority: sprite.has_priority(),
                    };
                }
            }
        }
        pixels
    }

    fn combine_pixels(&self, bg_pixels: [u8; COLS], window_pixels: [Option<u8>; COLS], sprite_pixels: [SpritePixel; COLS]) -> [u8; COLS] {
        let mut combined = [0; COLS];
        for (i, ((&bg, &window), &sprite)) in bg_pixels
            .iter()
            .zip(window_pixels.iter())
            .zip(sprite_pixels.iter())
            .enumerate()
        {
            let bg = match window {
                Some(x) => x,
                None => bg,
            };
            combined[i] = if bg != 0 && !sprite.priority {
                apply_palette(bg, Palette::BackgroundPalette, &self.state.palettes)
            } else if let Some(color) = sprite.pixel {
                apply_palette(color, sprite.palette.unwrap(), &self.state.palettes)
            } else {
                apply_palette(bg, Palette::BackgroundPalette, &self.state.palettes)
            }
        }
        combined
    }

    fn render_screen(&mut self) {
        self.display.redraw();
    }

    fn increment_current_line(&mut self) -> u8 {
        self.state.current_line += 1;
        self.check_fire_coincidence_interrupt();
        self.state.current_line
    }

    fn reset_current_line(&mut self) {
        self.state.current_line = 0;
        self.check_fire_coincidence_interrupt();
    }

    fn check_fire_coincidence_interrupt(&mut self) {
        if self.state.state_interrupt_lycly_coincidence && self.state.lyc == self.state.current_line {
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

#[derive(Copy, Clone)]
pub enum TileMap {
    Map0,
    Map1,
}

#[derive(Copy, Clone)]
pub enum TileSet {
    Set0,
    Set1,
}

