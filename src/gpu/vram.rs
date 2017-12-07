use memory::{BlockMemory, Memory};
use super::TileSet;
use super::constants::OFFSET_TILE_SET_0;
use super::constants::OFFSET_TILE_SET_1;
use super::constants::TILE_SIZE_IN_BYTES;

pub struct Vram {
    memory: BlockMemory,
}

impl Vram {
    pub fn new() -> Vram {
        Vram { memory: BlockMemory::new(0x2000) }
    }

    pub fn get_tile_row(&self, tile_set: TileSet, tile_num: u8, line_num: u8) -> TileRow {
        let base_offset;
        let tile_offset;
        match tile_set {
            TileSet::Set0 => {
                base_offset = OFFSET_TILE_SET_0;
                tile_offset = tile_num as i8 as i32 * TILE_SIZE_IN_BYTES as i32;
            }
            TileSet::Set1 => {
                base_offset = OFFSET_TILE_SET_1;
                tile_offset = tile_num as i32 * TILE_SIZE_IN_BYTES as i32;
            }
        };
        let address = (base_offset as i32 + tile_offset) as u16 + line_num as u16 * 0x2;
        TileRow {
            low_bits: self.memory.read_byte(address),
            high_bits: self.memory.read_byte(address + 1),
        }
    }
}

impl Memory for Vram {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory.write_byte(address, value)
    }
}

#[derive(Clone, Copy)]
pub struct TileRow {
    low_bits: u8,
    high_bits: u8,
}

impl TileRow {
    pub fn get_pixel(&self, x: u8) -> u8 {
        let mut pixel = 0;
        if self.low_bits & (0x80 >> x) != 0 {
            pixel += 1;
        }
        if self.high_bits & (0x80 >> x) != 0 {
            pixel += 2;
        }
        pixel
    }
}
