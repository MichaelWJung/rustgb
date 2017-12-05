use memory::Memory;
use gpu::*;
use gpu::sprite::SpriteAttribute;

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

    pub fn get_color(&self, x: u8, y: u8, vram: &Memory) -> u8 {
        let x = if self.x_flip { 7 - x } else { x };
        let y = if self.y_flip { 7 - y } else { y };
        self.get_pixel_from_memory(x, y, vram)
    }

    fn get_pixel_from_memory(&self, x: u8, y: u8, vram: &Memory) -> u8 {
        let address = self.get_line_address(y);
        let low_bits = vram.read_byte(address);
        let high_bits = vram.read_byte(address + 1);
        let mut pixel = 0;
        if low_bits & (0x80 >> x) != 0 {
            pixel += 1;
        }
        if high_bits & (0x80 >> x) != 0 {
            pixel += 2;
        }
        pixel
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

pub struct TileIterator<'a, M>
    where M: Memory + 'a
{
    bg_x: u8,
    x: u8,
    y: u8,
    tile_number: u8,
    tile_map: TileMap,
    vram: &'a M
}

impl<'a, M> TileIterator<'a, M>
    where M: Memory
{
    pub fn new(x: u8, y: u8, tile_map: TileMap, vram: &'a M) -> TileIterator<M> {
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

