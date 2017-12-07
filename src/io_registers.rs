use display::Display;
use gpu::{Gpu, TileMap, TileSet};
use memory::{BlockMemory, Memory};
use timer::{Timer, TimerSpeed};
use std::cell::RefCell;

const OFFSET_DIVIDER_REGISTER: u16 = 0x04;
const OFFSET_TIMER_COUNTER: u16 = 0x05;
const OFFSET_TIMER_MODULO: u16 = 0x06;
const OFFSET_TIMER_CONTROL: u16 = 0x07;
const OFFSET_INTERRUPT_FLAGS: u16 = 0x0F;
const OFFSET_LCD_CONTROL: u16 = 0x40;
const OFFSET_LCDC_STATUS: u16 = 0x41;
const OFFSET_SCY: u16 = 0x42;
const OFFSET_SCX: u16 = 0x43;
const OFFSET_LY: u16 = 0x44;
const OFFSET_LYC: u16 = 0x45;
const OFFSET_BACKGROUND_PALETTE: u16 = 0x47;
const OFFSET_OBJECT0_PALETTE: u16 = 0x48;
const OFFSET_OBJECT1_PALETTE: u16 = 0x49;

pub struct IoRegisters<'a, D>
    where D: Display + 'a
{
    old_io: BlockMemory,
    gpu: &'a RefCell<Gpu<D>>,
    timer: &'a RefCell<Timer>,
}

impl<'a, D> IoRegisters<'a, D>
    where D: Display
{
    pub fn new(gpu: &'a RefCell<Gpu<D>>, timer: &'a RefCell<Timer>) -> IoRegisters<'a, D> {
        IoRegisters {
            old_io: BlockMemory::new(0x80),
            gpu,
            timer,
        }
    }
}

impl <'a, D> Memory for IoRegisters<'a, D>
    where D: Display
{
    fn read_byte(&self, address: u16) -> u8 {
        let old_io = self.old_io.read_byte(address);
        match address {
            OFFSET_DIVIDER_REGISTER => self.timer.borrow().get_divider(),
            OFFSET_TIMER_COUNTER => self.timer.borrow().timer_counter,
            OFFSET_TIMER_MODULO => self.timer.borrow().timer_modulo,
            OFFSET_TIMER_CONTROL => {
                let timer = self.timer.borrow();
                let timer_speed = timer.timer_speed.to_u8();
                let timer_enabled = (timer.timer_enabled as u8) << 2;
                timer_speed | timer_enabled
            }
            OFFSET_INTERRUPT_FLAGS => {
                let gpu = self.gpu.borrow();
                let state = &gpu.state;
                let vblank_interrupt = state.vblank_interrupt_status as u8;
                let state_interrupt = (state.state_interrupt_status as u8) << 1;
                let timer_interrupt = (self.timer.borrow().timer_interrupt as u8) << 2;
                old_io & 0b1111_1000 | vblank_interrupt | state_interrupt | timer_interrupt
            }
            OFFSET_LCD_CONTROL => {
                let gpu = self.gpu.borrow();
                let state = &gpu.state;
                let bg_on = gpu.state.bg_on as u8;
                let sprites_on = (state.sprites_on as u8) << 1;
                let large_sprites = (state.large_sprites as u8) << 2;
                let bg_tile_map = (tile_map_to_bool(state.bg_tile_map) as u8) << 3;
                let bg_tile_set = (tile_set_to_bool(state.bg_tile_set) as u8) << 4;
                let window_on = (state.window_on as u8) << 5;
                let window_tile_map = (tile_map_to_bool(state.window_tile_map) as u8) << 6;
                let display_on = (state.get_display_on() as u8) << 7;
                bg_on | sprites_on | large_sprites | bg_tile_map | bg_tile_set
                      | window_on | window_tile_map | display_on
            }
            OFFSET_LCDC_STATUS => {
                let gpu = self.gpu.borrow();
                let state = &gpu.state;
                let mode_flag = state.get_mode();
                let hblank_interrupt = (state.state_interrupt_hblank as u8) << 3;
                let vblank_interrupt = (state.state_interrupt_vblank as u8) << 4;
                let oam_interrupt = (state.state_interrupt_oam as u8) << 5;
                let lycly_coincidence_interrupt = (state.state_interrupt_lycly_coincidence as u8) << 6;
                old_io & 0b1000_0100 | mode_flag
                                     | hblank_interrupt
                                     | vblank_interrupt
                                     | oam_interrupt
                                     | lycly_coincidence_interrupt
            }
            OFFSET_SCY => self.gpu.borrow().state.scy,
            OFFSET_SCX => self.gpu.borrow().state.scx,
            OFFSET_LY => self.gpu.borrow().state.get_current_line(),
            OFFSET_LYC => self.gpu.borrow().state.lyc,
            OFFSET_BACKGROUND_PALETTE => self.gpu.borrow().state.palettes.bg,
            OFFSET_OBJECT0_PALETTE => self.gpu.borrow().state.palettes.obj0,
            OFFSET_OBJECT1_PALETTE => self.gpu.borrow().state.palettes.obj1,
            _ => old_io,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            OFFSET_DIVIDER_REGISTER => self.timer.borrow_mut().reset_divider(),
            OFFSET_TIMER_COUNTER => self.timer.borrow_mut().timer_counter = value,
            OFFSET_TIMER_MODULO => self.timer.borrow_mut().timer_modulo = value,
            OFFSET_TIMER_CONTROL => {
                let timer_speed = value & 0b0000_0011;
                let timer_enabled = value & 0b0000_0100 != 0;
                let mut timer = self.timer.borrow_mut();
                timer.timer_speed = TimerSpeed::from_u8(timer_speed);
                timer.timer_enabled = timer_enabled;

            }
            OFFSET_INTERRUPT_FLAGS => {
                self.gpu.borrow_mut().state.vblank_interrupt_status = value & 1 != 0;
                self.gpu.borrow_mut().state.state_interrupt_status = value & 2 != 0;
                self.timer.borrow_mut().timer_interrupt = value & 4 != 0;
            }
            OFFSET_LCDC_STATUS => {
                let hblank_interrupt = value & 0b0000_1000 != 0;
                let vblank_interrupt = value & 0b0001_0000 != 0;
                let oam_interrupt = value & 0b0010_0000 != 0;
                let lycly_coincidence_interrupt = value & 0b0100_0000 != 0;
                let mut gpu = self.gpu.borrow_mut();
                let state = &mut gpu.state;
                state.state_interrupt_hblank = hblank_interrupt;
                state.state_interrupt_vblank = vblank_interrupt;
                state.state_interrupt_oam = oam_interrupt;
                state.state_interrupt_lycly_coincidence = lycly_coincidence_interrupt;
            }
            OFFSET_LCD_CONTROL => {
                let bg_on = value & 0b0000_0001 != 0;
                let sprites_on = value & 0b0000_0010 != 0;
                let large_sprites = value & 0b0000_0100 != 0;
                let bg_tile_map = value & 0b0000_1000 != 0;
                let bg_tile_set = value & 0b0001_0000 != 0;
                let window_on = value & 0b0010_0000 != 0;
                let window_tile_map = value & 0b0100_0000 != 0;
                let display_on = value & 0b1000_0000 != 0;
                let mut gpu = self.gpu.borrow_mut();
                let state = &mut gpu.state;
                state.bg_on = bg_on;
                state.sprites_on = sprites_on;
                state.large_sprites = large_sprites;
                state.bg_tile_map = bool_to_tile_map(bg_tile_map);
                state.bg_tile_set = bool_to_tile_set(bg_tile_set);
                state.window_on = window_on;
                state.window_tile_map = bool_to_tile_map(window_tile_map);
                if state.get_display_on() != display_on {
                    state.set_display_on(display_on);
                }
            }
            OFFSET_SCY => self.gpu.borrow_mut().state.scy = value,
            OFFSET_SCX => self.gpu.borrow_mut().state.scx = value,
            OFFSET_LY => (),
            OFFSET_LYC => self.gpu.borrow_mut().state.lyc = value,
            OFFSET_BACKGROUND_PALETTE => self.gpu.borrow_mut().state.palettes.bg = value,
            OFFSET_OBJECT0_PALETTE => self.gpu.borrow_mut().state.palettes.obj0 = value,
            OFFSET_OBJECT1_PALETTE => self.gpu.borrow_mut().state.palettes.obj1 = value,
            _ => self.old_io.write_byte(address, value),
        }
    }
}

fn bool_to_tile_set(b: bool) -> TileSet {
    if b { TileSet::Set1 } else { TileSet::Set0 }
}

fn tile_set_to_bool(tile_set: TileSet) -> bool {
    match tile_set {
        TileSet::Set0 => false,
        TileSet::Set1 => true,
    }
}

fn bool_to_tile_map(b: bool) -> TileMap {
    if b { TileMap::Map1 } else { TileMap::Map0 }
}

fn tile_map_to_bool(tile_map: TileMap) -> bool {
    match tile_map {
        TileMap::Map0 => false,
        TileMap::Map1 => true,
    }
}
