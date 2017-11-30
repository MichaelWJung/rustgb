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

    pub fn get_color<M: Memory>(&self, x: u8, y: u8, vram: &M) -> u8 {
        let x = if self.x_flip { 7 - x } else { x };
        let y = if self.y_flip { 7 - y } else { y };
        let line = self.get_line(y, vram);
        line[x as usize]
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

pub struct TileIterator<'a, M>
    where M: Memory + 'a
{
    bg_x: u8,
    pub x: u8,
    pub y: u8,
    pub tile_number: u8,
    tile_map: &'a TileMap,
    vram: &'a M
}

impl<'a, M> TileIterator<'a, M>
    where M: Memory
{
    pub fn next(&mut self) {
        self.x = (self.x + 1) % 8;
        self.bg_x = self.bg_x.wrapping_add(1);
        if self.x == 0 {
            *self = self.tile_map.get_tile_iter(self.bg_x, self.y, self.vram);
        }
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

    pub fn get_tile_iter<'a, 'b: 'a, M: Memory>(&'a self, x: u8, y: u8, vram: &'b M) -> TileIterator<M> {
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


