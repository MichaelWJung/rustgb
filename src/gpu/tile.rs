use memory::Memory;
use gpu::*;

#[derive(Copy, Clone)]
pub struct Tile {
    pub tile_num: u8,
    pub tile_set: TileSet,
    pub x_flip: bool,
    pub y_flip: bool,
    pub _large_tile: bool,
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
    pub x: u8,
    pub y: u8,
    pub tile_number: u8,
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
}

