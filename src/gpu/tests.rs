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
        let mut gpu = Gpu::new(display);
        {
            gpu.palettes.bg = 0b11100100;
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
        let mut gpu = Gpu::new(display);
        {
            gpu.palettes.bg = 0b11100100;
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
        let mut gpu = Gpu::new(display);
        {
            gpu.palettes.bg = 0b11100100;
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

