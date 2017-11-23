use display::Display;
use gpu::Gpu;
use memory::{BlockMemory, Memory};
use std::cell::RefCell;

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
            OFFSET_SCY => self.gpu.borrow_mut().scy = value,
            OFFSET_SCX => self.gpu.borrow_mut().scx = value,
            OFFSET_LY => (),
            _ => self.old_io.borrow_mut().write_byte(address, value),
        }
    }
}
