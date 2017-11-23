use display::Display;
use gpu::{Gpu, TileMap};
use memory::{BlockMemory, Memory};
use std::cell::RefCell;

const OFFSET_LCD_CONTROL: u16 = 0x40;
const OFFSET_LCDC_STATUS: u16 = 0x41;
const OFFSET_SCY: u16 = 0x42;
const OFFSET_SCX: u16 = 0x43;
const OFFSET_LY: u16 = 0x44;

pub struct IoRegisters<'a, 'b, D>
    where 'b : 'a,
          D: Display + 'b
{
    old_io: &'a RefCell<BlockMemory>,
    gpu: &'a RefCell<Gpu<'b, D>>,
}

impl<'a, 'b, D> IoRegisters<'a, 'b, D>
    where D: Display
{
    pub fn new(old_io: &'a RefCell<BlockMemory>, gpu: &'a RefCell<Gpu<'b, D>>) -> IoRegisters<'a, 'b, D> {
        IoRegisters {
            old_io,
            gpu,
        }
    }
}

impl <'a, 'b, D> Memory for IoRegisters<'a, 'b, D>
    where D: Display
{
    fn read_byte(&self, address: u16) -> u8 {
        let old_io = self.old_io.borrow().read_byte(address);
        match address {
            OFFSET_LCD_CONTROL => {
                let gpu = self.gpu.borrow();
                let bg_on = gpu.bg_on as u8;
                let sprites_on = (gpu.sprites_on as u8) << 1;
                let large_sprites = (gpu.large_sprites as u8) << 2;
                let bg_tile_map = (gpu.bg_tile_map.to_bool() as u8) << 3;
                old_io & 0b1111_1110 | bg_on
                                     | sprites_on
                                     | large_sprites
                                     | bg_tile_map
            }
            OFFSET_LCDC_STATUS => {
                (old_io & 0b1111_1100) | self.gpu.borrow().get_mode()
            }
            OFFSET_SCY => self.gpu.borrow().scy,
            OFFSET_SCX => self.gpu.borrow().scx,
            OFFSET_LY => self.gpu.borrow().get_current_line(),
            _ => old_io,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            OFFSET_LCD_CONTROL => {
                self.old_io.borrow_mut().write_byte(address, value);
                let mut gpu = self.gpu.borrow_mut();
                let bg_on = value & 0b0000_0001 != 0;
                let sprites_on = value & 0b0000_0010 != 0;
                let large_sprites = value & 0b0000_0100 != 0;
                let bg_tile_map = value & 0b0000_1000 != 0;
                gpu.bg_on = bg_on;
                gpu.sprites_on = sprites_on;
                gpu.large_sprites = large_sprites;
                gpu.bg_tile_map = TileMap::from_bool(bg_tile_map);
            }
            OFFSET_SCY => self.gpu.borrow_mut().scy = value,
            OFFSET_SCX => self.gpu.borrow_mut().scx = value,
            OFFSET_LY => (),
            _ => self.old_io.borrow_mut().write_byte(address, value),
        }
    }
}
