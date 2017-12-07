use memory::Memory;
use super::*;
use super::sprite::SpriteAttribute;
use super::vram::Vram;

#[derive(Copy, Clone)]
pub struct Tile {
    tile_num: u8,
    tile_set: TileSet,
    x_flip: bool,
    y_flip: bool,
    _large_tile: bool,
}

impl Tile {
    pub fn new(tile_num: u8, tile_set: TileSet) -> Tile {
        Tile {
            tile_num,
            tile_set,
            x_flip: false,
            y_flip: false,
            _large_tile: false,
        }
    }

    pub fn from_sprite(sprite: &SpriteAttribute) -> Tile {
        Tile {
            tile_num: sprite.get_tile_num(),
            tile_set: TileSet::Set1,
            x_flip: sprite.has_x_flip(),
            y_flip: sprite.has_y_flip(),
            _large_tile: false,
        }
    }

    pub fn get_color(&self, x: u8, y: u8, vram: &Vram) -> u8 {
        let x = if self.x_flip { 7 - x } else { x };
        let y = if self.y_flip { 7 - y } else { y };
        let tile_row = vram.get_tile_row(self.tile_set, self.tile_num, y);
        tile_row.get_pixel(x)
    }
}

pub struct TileIterator<'a>
{
    bg_x: u8,
    x: u8,
    y: u8,
    tile_number: u8,
    tile_map: TileMap,
    vram: &'a Vram
}

impl<'a> TileIterator<'a>
{
    pub fn new(x: u8, y: u8, tile_map: TileMap, vram: &'a Vram) -> TileIterator {
        let base_offset = match tile_map {
            TileMap::Map0 => OFFSET_TILE_MAP_0,
            TileMap::Map1 => OFFSET_TILE_MAP_1,
        };
        let row = y / 8;
        let col = x / 8;
        let tile_offset = row as u16 * 32 + col as u16;
        let tile_number = vram.read_byte(base_offset + tile_offset);
        TileIterator { bg_x: x, x: x % 8, y, tile_number, tile_map, vram }
    }

    pub fn next(&mut self) {
        self.x = (self.x + 1) % 8;
        self.bg_x = self.bg_x.wrapping_add(1);
        if self.x == 0 {
            *self = Self::new(self.bg_x, self.y, self.tile_map, &self.vram);
        }
    }

    pub fn get_pixel_color(&self, tile_set: TileSet) -> u8 {
        let tile = Tile::new(self.tile_number, tile_set);
        tile.get_color(self.x as u8 % 8, self.y as u8 % 8, self.vram)
    }
}

