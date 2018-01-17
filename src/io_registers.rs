use apu::Apu;
use display::Display;
use gpu::{Gpu, TileMap, TileSet};
use memory::{BlockMemory, Memory};
use timer::{Timer, TimerSpeed};
use std::cell::RefCell;

const OFFSET_JOYPAD: u16 = 0x00;
const OFFSET_SERIAL_TRANSFER_CONTROL: u16 = 0x02;
const OFFSET_DIVIDER_REGISTER: u16 = 0x04;
const OFFSET_TIMER_COUNTER: u16 = 0x05;
const OFFSET_TIMER_MODULO: u16 = 0x06;
const OFFSET_TIMER_CONTROL: u16 = 0x07;
const OFFSET_INTERRUPT_FLAGS: u16 = 0x0F;
const OFFSET_CHANNEL_1_SWEEP_REGISTER: u16 = 0x10;
const OFFSET_CHANNEL_1_LENGTH_DUTY: u16 = 0x11;
const OFFSET_CHANNEL_1_VOLUME_ENVELOPE: u16 = 0x12;
const OFFSET_CHANNEL_1_FREQUENCY_LO: u16 = 0x13;
const OFFSET_CHANNEL_1_FREQUENCY_HI: u16 = 0x14;
const OFFSET_CHANNEL_2_LENGTH_DUTY: u16 = 0x16;
const OFFSET_CHANNEL_2_VOLUME_ENVELOPE: u16 = 0x17;
const OFFSET_CHANNEL_2_FREQUENCY_LO: u16 = 0x18;
const OFFSET_CHANNEL_2_FREQUENCY_HI: u16 = 0x19;
const OFFSET_CHANNEL_3_SOUND_ON_OFF: u16 = 0x1A;
const OFFSET_CHANNEL_3_SOUND_LENGTH: u16 = 0x1B;
const OFFSET_CHANNEL_3_SELECT_OUTPUT_LEVEL: u16 = 0x1C;
const OFFSET_CHANNEL_3_FREQUENCY_LO: u16 = 0x1D;
const OFFSET_CHANNEL_3_FREQUENCY_HI: u16 = 0x1E;
const OFFSET_CHANNEL_4_SOUND_LENGTH: u16 = 0x20;
const OFFSET_CHANNEL_4_VOLUME_ENVELOPE: u16 = 0x21;
const OFFSET_CHANNEL_4_POLYNOMIAL_COUNTER: u16 = 0x22;
const OFFSET_CHANNEL_4_COUNTER_CONSECUTIVE_INITIAL: u16 = 0x23;
const OFFSET_SOUND_ON_OFF: u16 = 0x26;
const OFFSET_BEGIN_WAVE_PATTERN_RAM: u16 = 0x30;
const OFFSET_END_WAVE_PATTERN_RAM: u16 = 0x3F;
const OFFSET_LCD_CONTROL: u16 = 0x40;
const OFFSET_LCDC_STATUS: u16 = 0x41;
const OFFSET_SCY: u16 = 0x42;
const OFFSET_SCX: u16 = 0x43;
const OFFSET_LY: u16 = 0x44;
const OFFSET_LYC: u16 = 0x45;
const OFFSET_WINDOW_Y: u16 = 0x4A;
const OFFSET_WINDOW_X: u16 = 0x4B;
const OFFSET_BACKGROUND_PALETTE: u16 = 0x47;
const OFFSET_OBJECT0_PALETTE: u16 = 0x48;
const OFFSET_OBJECT1_PALETTE: u16 = 0x49;

pub struct IoRegisters<'a, D>
where
    D: Display + 'a,
{
    old_io: BlockMemory,
    apu: &'a RefCell<Apu<'a>>,
    gpu: &'a RefCell<Gpu<D>>,
    timer: &'a RefCell<Timer>,
}

impl<'a, D> IoRegisters<'a, D>
where
    D: Display,
{
    pub fn new(
        apu: &'a RefCell<Apu<'a>>,
        gpu: &'a RefCell<Gpu<D>>,
        timer: &'a RefCell<Timer>,
    ) -> IoRegisters<'a, D> {
        IoRegisters {
            old_io: BlockMemory::new(0x80),
            apu,
            gpu,
            timer,
        }
    }
}

impl<'a, D> Memory for IoRegisters<'a, D>
where
    D: Display,
{
    fn read_byte(&self, address: u16) -> u8 {
        let old_io = self.old_io.read_byte(address);
        match address {
            OFFSET_JOYPAD => old_io | 0b1100_0000, // bits 6 and 7 unused
            OFFSET_SERIAL_TRANSFER_CONTROL => old_io | 0b0111_1110, // bits 1-6 unused
            OFFSET_DIVIDER_REGISTER => self.timer.borrow().get_divider(),
            OFFSET_TIMER_COUNTER => self.timer.borrow().timer_counter,
            OFFSET_TIMER_MODULO => self.timer.borrow().timer_modulo,
            OFFSET_TIMER_CONTROL => {
                let timer = self.timer.borrow();
                let timer_speed = timer.timer_speed.to_u8();
                let timer_enabled = (timer.timer_enabled as u8) << 2;
                timer_speed | timer_enabled | 0b1111_1000 // bits 3-7 unused
            }
            OFFSET_INTERRUPT_FLAGS => {
                let gpu = self.gpu.borrow();
                let state = &gpu.state;
                let vblank_interrupt = state.vblank_interrupt_status as u8;
                let state_interrupt = (state.state_interrupt_status as u8) << 1;
                let timer_interrupt = (self.timer.borrow().timer_interrupt as u8) << 2;
                old_io & 0b0001_1000 | vblank_interrupt | state_interrupt | timer_interrupt |
                    0b1110_0000 // bits 5-7 unused
            }
            OFFSET_CHANNEL_1_SWEEP_REGISTER => old_io | 0b1000_0000, // bit 7 unused
            OFFSET_CHANNEL_1_LENGTH_DUTY => {
                let duty = self.apu.borrow().get_channel1_duty() << 6;
                duty | 0b0011_1111
            }
            OFFSET_CHANNEL_1_VOLUME_ENVELOPE => {
                let apu = self.apu.borrow();
                let starting_volume = (apu.get_channel1_envelope_starting_volume() & 0xF) << 4;
                let envelope_direction = (apu.get_channel1_volume_envelope_direction() as u8) << 3;
                let envelope_period = apu.get_channel1_volume_envelope_period() & 0b0000_0111;
                starting_volume | envelope_direction | envelope_period
            }
            // TODO: Looks like frequency_lo cannot be read
            OFFSET_CHANNEL_1_FREQUENCY_LO => self.apu.borrow().get_channel1_frequency_lo(),
            OFFSET_CHANNEL_1_FREQUENCY_HI => {
                let apu = self.apu.borrow();
                // TODO: Looks like frequency_hi cannot be read
                let frequency_hi = apu.get_channel1_frequency_hi() & 0b0000_0111;
                // TODO: Counter_on is on bit 6, bit 7 cannot be read
                let counter_on = (apu.get_channel1_counter_on() as u8) << 7;
                frequency_hi | counter_on | 0b1011_1000
            }
            OFFSET_CHANNEL_2_LENGTH_DUTY => {
                let duty = self.apu.borrow().get_channel2_duty() << 6;
                duty | 0b0011_1111
            }
            OFFSET_CHANNEL_2_VOLUME_ENVELOPE => {
                let apu = self.apu.borrow();
                let starting_volume = (apu.get_channel2_envelope_starting_volume() & 0xF) << 4;
                let envelope_direction = (apu.get_channel2_volume_envelope_direction() as u8) << 3;
                let envelope_period = apu.get_channel2_volume_envelope_period() & 0b0000_0111;
                starting_volume | envelope_direction | envelope_period
            }
            // TODO: Looks like frequency_lo cannot be read
            OFFSET_CHANNEL_2_FREQUENCY_LO => self.apu.borrow().get_channel2_frequency_lo(),
            OFFSET_CHANNEL_2_FREQUENCY_HI => {
                let apu = self.apu.borrow();
                // TODO: Looks like frequency_hi cannot be read
                let frequency_hi = apu.get_channel2_frequency_hi() & 0b0000_0111;
                // TODO: Counter_on is on bit 6, bit 7 cannot be read
                let counter_on = (apu.get_channel2_counter_on() as u8) << 7;
                frequency_hi | counter_on | 0b1011_1000
            }
            OFFSET_CHANNEL_3_SOUND_ON_OFF => {
                let sound_on = (self.apu.borrow().get_channel3_on() as u8) << 7;
                sound_on | 0b0111_1111 // bits 0-6 unused
            }
            OFFSET_CHANNEL_3_SOUND_LENGTH => old_io,
            OFFSET_CHANNEL_3_SELECT_OUTPUT_LEVEL => old_io | 0b1001_1111, // bits 0-4,7 unused
            // TODO: Looks like frequency_lo cannot be read
            OFFSET_CHANNEL_3_FREQUENCY_LO => self.apu.borrow().get_channel3_frequency_lo(),
            OFFSET_CHANNEL_3_FREQUENCY_HI => {
                let apu = self.apu.borrow();
                let frequency_hi = apu.get_channel3_frequency_hi() & 0b0000_0111;
                let counter_on = (apu.get_channel3_counter_on() as u8) << 6;
                // TODO: Looks like frequency_hi cannot be read
                frequency_hi | counter_on | 0b1011_1000
            }
            OFFSET_CHANNEL_4_SOUND_LENGTH => old_io | 0b1100_0000, // bits 6-7 unused
            OFFSET_CHANNEL_4_VOLUME_ENVELOPE => {
                let apu = self.apu.borrow();
                let starting_volume = (apu.get_channel4_envelope_starting_volume() & 0xF) << 4;
                let envelope_direction = (apu.get_channel4_volume_envelope_direction() as u8) << 3;
                let envelope_period = apu.get_channel4_volume_envelope_period() & 0b0000_0111;
                starting_volume | envelope_direction | envelope_period
            }
            OFFSET_CHANNEL_4_POLYNOMIAL_COUNTER => {
                let apu = self.apu.borrow();
                let prescaler_divider = (apu.get_channel4_prescaler_divider() & 0xF) << 4;
                let shift_register_width = (apu.get_channel4_shift_register_width() as u8) << 3;
                let clock_divider = apu.get_channel4_clock_divider() & 0x7;
                prescaler_divider | shift_register_width | clock_divider
            }
            OFFSET_CHANNEL_4_COUNTER_CONSECUTIVE_INITIAL => {
                let counter_on = (self.apu.borrow().get_channel4_counter_on() as u8) << 6;
                counter_on | 0b1011_1111 // bits 0-5 unused, bit 7 write only
            }
            OFFSET_SOUND_ON_OFF => {
                let apu = self.apu.borrow();
                let sound_on = (apu.get_sound_on() as u8) << 7;
                let channel1_on = apu.get_channel1_on() as u8;
                let channel2_on = (apu.get_channel2_on() as u8) << 1;
                sound_on | channel1_on | channel2_on | 0b0111_0000 // buts 4-6 unused
            }
            OFFSET_BEGIN_WAVE_PATTERN_RAM...OFFSET_END_WAVE_PATTERN_RAM => {
                let offset = (address - OFFSET_BEGIN_WAVE_PATTERN_RAM) as usize * 2;
                let apu = self.apu.borrow();
                let wave_pattern = apu.channel3_wave_pattern();
                let first = wave_pattern[offset] & 0xF;
                let second = wave_pattern[offset + 1] & 0xF;
                (first << 4) | second
            }
            OFFSET_LCD_CONTROL => {
                let gpu = self.gpu.borrow();
                let state = &gpu.state;
                let bg_on = gpu.state.bg_on as u8;
                let sprites_on = (state.sprites_on as u8) << 1;
                let large_sprites = (state.large_sprites as u8) << 2;
                let bg_tile_map = (tile_map_to_bool(state.bg_tile_map) as u8) << 3;
                let bg_window_tile_set = (tile_set_to_bool(state.bg_window_tile_set) as u8) << 4;
                let window_on = (state.window_on as u8) << 5;
                let window_tile_map = (tile_map_to_bool(state.window_tile_map) as u8) << 6;
                let display_on = (state.get_display_on() as u8) << 7;
                bg_on | sprites_on | large_sprites | bg_tile_map | bg_window_tile_set |
                    window_on | window_tile_map | display_on
            }
            OFFSET_LCDC_STATUS => {
                let gpu = self.gpu.borrow();
                let state = &gpu.state;
                let mode_flag = state.get_mode();
                let hblank_interrupt = (state.state_interrupt_hblank as u8) << 3;
                let vblank_interrupt = (state.state_interrupt_vblank as u8) << 4;
                let oam_interrupt = (state.state_interrupt_oam as u8) << 5;
                let lycly_coincidence_interrupt = (state.state_interrupt_lycly_coincidence as u8) <<
                    6;
                old_io & 0b0000_0100 | mode_flag | hblank_interrupt | vblank_interrupt |
                    oam_interrupt | lycly_coincidence_interrupt | 0b1000_0000 // bit 7 unused
            }
            OFFSET_SCY => self.gpu.borrow().state.scy,
            OFFSET_SCX => self.gpu.borrow().state.scx,
            OFFSET_LY => self.gpu.borrow().state.get_current_line(),
            OFFSET_LYC => self.gpu.borrow().state.lyc,
            OFFSET_WINDOW_Y => self.gpu.borrow().state.window_y,
            OFFSET_WINDOW_X => self.gpu.borrow().state.window_x,
            OFFSET_BACKGROUND_PALETTE => self.gpu.borrow().state.palettes.bg,
            OFFSET_OBJECT0_PALETTE => self.gpu.borrow().state.palettes.obj0,
            OFFSET_OBJECT1_PALETTE => self.gpu.borrow().state.palettes.obj1,
            // Completely unused bytes
            0x03 | 0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x15 | 0x1F | 0x27 | 0x28 |
            0x29 | 0x4C...0x7F => 0xFF,
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
            OFFSET_CHANNEL_1_LENGTH_DUTY => {
                let length = value & 0b0011_1111;
                let duty = (value & 0b1100_0000) >> 6;
                let mut apu = self.apu.borrow_mut();
                apu.set_channel1_counter(64 - length);
                apu.set_channel1_duty(duty);
            }
            OFFSET_CHANNEL_1_VOLUME_ENVELOPE => {
                let starting_volume = (value & 0b1111_0000) >> 4;
                let envelope_direction = value & 0b0000_1000 != 0;
                let envelope_period = value & 0b0000_0111;
                let mut apu = self.apu.borrow_mut();
                apu.set_channel1_envelope_starting_volume(starting_volume);
                apu.set_channel1_volume_envelope_direction(
                    envelope_direction,
                );
                apu.set_channel1_volume_envelope_period(envelope_period);
            }
            OFFSET_CHANNEL_1_FREQUENCY_LO => self.apu.borrow_mut().set_channel1_frequency_lo(value),
            OFFSET_CHANNEL_1_FREQUENCY_HI => {
                let mut apu = self.apu.borrow_mut();
                if value & 0b1000_0000 != 0 {
                    apu.restart_channel1_sound();
                }
                // TODO: Should be bit 6
                apu.set_channel1_counter_on(value & 0b1000_0000 != 0);
                apu.set_channel1_frequency_hi(value & 0b0000_0111);
            }
            OFFSET_CHANNEL_2_LENGTH_DUTY => {
                let length = value & 0b0011_1111;
                let duty = (value & 0b1100_0000) >> 6;
                let mut apu = self.apu.borrow_mut();
                apu.set_channel2_counter(64 - length);
                apu.set_channel2_duty(duty);
            }
            OFFSET_CHANNEL_2_VOLUME_ENVELOPE => {
                let starting_volume = (value & 0b1111_0000) >> 4;
                let envelope_direction = value & 0b0000_1000 != 0;
                let envelope_period = value & 0b0000_0111;
                let mut apu = self.apu.borrow_mut();
                apu.set_channel2_envelope_starting_volume(starting_volume);
                apu.set_channel2_volume_envelope_direction(
                    envelope_direction,
                );
                apu.set_channel2_volume_envelope_period(envelope_period);
            }
            OFFSET_CHANNEL_2_FREQUENCY_LO => self.apu.borrow_mut().set_channel2_frequency_lo(value),
            OFFSET_CHANNEL_2_FREQUENCY_HI => {
                let mut apu = self.apu.borrow_mut();
                if value & 0b1000_0000 != 0 {
                    apu.restart_channel2_sound();
                }
                // TODO: Should be bit 6
                apu.set_channel2_counter_on(value & 0b1000_0000 != 0);
                apu.set_channel2_frequency_hi(value & 0b0000_0111);
            }
            OFFSET_CHANNEL_3_SOUND_ON_OFF => {
                let sound_on = value & 0b1000_0000 != 0;
                self.apu.borrow_mut().set_channel3_on(sound_on);
            }
            OFFSET_CHANNEL_3_SOUND_LENGTH => {
                self.apu.borrow_mut().set_channel3_counter(
                    256 - value as u16,
                );
                self.old_io.write_byte(address, value);
            }
            OFFSET_CHANNEL_3_SELECT_OUTPUT_LEVEL => {
                let volume = (value & 0b0110_0000) >> 5;
                let volume = match volume {
                    0 => 0,
                    1 => 3,
                    2 => 2,
                    3 => 1,
                    _ => panic!("Code location should be unreachable"),
                };
                self.apu.borrow_mut().set_channel3_volume(volume);
                self.old_io.write_byte(address, value);
            }
            OFFSET_CHANNEL_3_FREQUENCY_LO => self.apu.borrow_mut().set_channel3_frequency_lo(value),
            OFFSET_CHANNEL_3_FREQUENCY_HI => {
                let mut apu = self.apu.borrow_mut();
                if value & 0b1000_0000 != 0 {
                    apu.restart_channel3_sound();
                }
                apu.set_channel3_counter_on(value & 0b0100_0000 != 0);
                apu.set_channel3_frequency_hi(value & 0b0000_0111);
            }
            OFFSET_CHANNEL_4_SOUND_LENGTH => {
                self.apu.borrow_mut().set_channel4_counter(
                    64 - (value & 0b0011_1111),
                );
                self.old_io.write_byte(address, value);
            }
            OFFSET_CHANNEL_4_VOLUME_ENVELOPE => {
                let starting_volume = (value & 0b1111_0000) >> 4;
                let envelope_direction = value & 0b0000_1000 != 0;
                let envelope_period = value & 0b0000_0111;
                let mut apu = self.apu.borrow_mut();
                apu.set_channel4_envelope_starting_volume(starting_volume);
                apu.set_channel4_volume_envelope_direction(
                    envelope_direction,
                );
                apu.set_channel4_volume_envelope_period(envelope_period);
            }
            OFFSET_CHANNEL_4_POLYNOMIAL_COUNTER => {
                let prescaler_divider = (value & 0b1111_0000) >> 4;
                let shift_register_width = value & 0b0000_1000 != 0;
                let clock_divider = value & 0b0000_0111;
                let mut apu = self.apu.borrow_mut();
                apu.set_channel4_prescaler_divider(prescaler_divider);
                apu.set_channel4_shift_register_width(shift_register_width);
                apu.set_channel4_clock_divider(clock_divider);
            }
            OFFSET_CHANNEL_4_COUNTER_CONSECUTIVE_INITIAL => {
                let mut apu = self.apu.borrow_mut();
                if value & 0b1000_0000 != 0 {
                    apu.restart_channel4_sound();
                }
                apu.set_channel4_counter_on(value & 0b0100_0000 != 0);
            }
            OFFSET_SOUND_ON_OFF => {
                self.apu.borrow_mut().set_sound_on(value & 0b1000_0000 != 0);
            }
            OFFSET_BEGIN_WAVE_PATTERN_RAM...OFFSET_END_WAVE_PATTERN_RAM => {
                let offset = (address - OFFSET_BEGIN_WAVE_PATTERN_RAM) as usize * 2;
                let first = (value & 0xF0) >> 4;
                let second = value & 0xF;
                let mut apu = self.apu.borrow_mut();
                let wave_pattern = apu.channel3_wave_pattern_mut();
                wave_pattern[offset] = first;
                wave_pattern[offset + 1] = second;
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
                let bg_window_tile_set = value & 0b0001_0000 != 0;
                let window_on = value & 0b0010_0000 != 0;
                let window_tile_map = value & 0b0100_0000 != 0;
                let display_on = value & 0b1000_0000 != 0;
                let mut gpu = self.gpu.borrow_mut();
                let state = &mut gpu.state;
                state.bg_on = bg_on;
                state.sprites_on = sprites_on;
                state.large_sprites = large_sprites;
                state.bg_tile_map = bool_to_tile_map(bg_tile_map);
                state.bg_window_tile_set = bool_to_tile_set(bg_window_tile_set);
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
            OFFSET_WINDOW_Y => self.gpu.borrow_mut().state.window_y = value,
            OFFSET_WINDOW_X => self.gpu.borrow_mut().state.window_x = value,
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
