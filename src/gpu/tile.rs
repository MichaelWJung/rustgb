use super::*;
use super::sprite::SpriteAttribute;
use super::vram::Vram;

#[derive(Copy, Clone)]
pub struct Tile {
    tile_num: u8,
    tile_set: TileSet,
    x_flip: bool,
    y_flip: bool,
    large_tile: bool,
}

impl Tile {
    pub fn new(tile_num: u8, tile_set: TileSet) -> Tile {
        Tile {
            tile_num,
            tile_set,
            x_flip: false,
            y_flip: false,
            large_tile: false,
        }
    }

    pub fn from_sprite(sprite: &SpriteAttribute) -> Tile {
        Tile {
            tile_num: sprite.get_tile_num(),
            tile_set: TileSet::Set1,
            x_flip: sprite.has_x_flip(),
            y_flip: sprite.has_y_flip(),
            large_tile: sprite.is_large_sprite(),
        }
    }

    pub fn get_color(&self, x: u8, y: u8, vram: &Vram) -> u8 {
        let x = if self.x_flip { 7 - x } else { x };
        let y = if self.y_flip {
            if self.large_tile {
                15 - y
            } else {
                7 - y
            }
        } else {
            y
        };
        let tile_row = vram.get_tile_row(self.tile_set, self.tile_num, y);
        tile_row.get_pixel(x)
    }
}

pub struct TileIterator<'a> {
    bg_x: u8,
    tile_x: u8,
    tile_y: u8,
    tile_num: u8,
    tile_map: TileMap,
    vram: &'a Vram,
}

impl<'a> TileIterator<'a> {
    pub fn new(x: u8, y: u8, tile_map: TileMap, vram: &'a Vram) -> TileIterator {
        let row = y / 8;
        let col = x / 8;
        let tile_num = vram.get_tile_num(tile_map, row, col);
        TileIterator {
            bg_x: x,
            tile_x: x % 8,
            tile_y: y,
            tile_num,
            tile_map,
            vram,
        }
    }

    pub fn next(&mut self) {
        self.tile_x = (self.tile_x + 1) % 8;
        self.bg_x = self.bg_x.wrapping_add(1);
        if self.tile_x == 0 {
            *self = Self::new(self.bg_x, self.tile_y, self.tile_map, &self.vram);
        }
    }

    pub fn get_pixel_color(&self, tile_set: TileSet) -> u8 {
        let tile = Tile::new(self.tile_num, tile_set);
        tile.get_color(self.tile_x as u8 % 8, self.tile_y as u8 % 8, self.vram)
    }
}
