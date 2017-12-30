use super::SOUND_SAMPLE_RATE_IN_HERTZ;
const FREQUENCY_TIMER_TICKS_PER_PERIOD: u32 = 1;

enum ShiftRegisterWidth {
    SevenBit,
    FifteenBit,
}

pub struct NoiseChannel {
    frequency_timer_ticks: u32,
    frequency_timer_tick_counts: u32,
    clock_divider: u8,
    prescaler_divider: u8,
    on: bool,
    counter_on: bool,
    counter: u8,
    volume: u8,
    envelope_starting_volume: u8,
    volume_envelope_direction: bool,
    volume_envelope_period: u8,
    volume_envelope_tick_counts: u8,
    shift_register: u16,
    shift_register_width: ShiftRegisterWidth,
}

impl NoiseChannel {
    pub fn new() -> NoiseChannel {
        // Default: 200 Hz
        let frequency_timer_ticks = Self::calc_num_timer_ticks_for_frequency(200);
        NoiseChannel {
            frequency_timer_ticks,
            frequency_timer_tick_counts: frequency_timer_ticks,
            clock_divider: 0,
            prescaler_divider: 0,
            on: false,
            counter_on: false,
            counter: 0,
            volume: 15,
            envelope_starting_volume: 0,
            volume_envelope_direction: false,
            volume_envelope_period: 0,
            volume_envelope_tick_counts: 0,
            shift_register: 0x7FFF,
            shift_register_width: ShiftRegisterWidth::FifteenBit,
        }
    }

    pub fn set_clock_divider(&mut self, clock_divider: u8) {
        self.clock_divider = clock_divider;
        self.set_frequency();
    }

    pub fn set_prescaler_divider(&mut self, prescaler_divider: u8) {
        self.prescaler_divider = prescaler_divider;
        self.set_frequency();
    }

    pub fn get_clock_divider(&self) -> u8 {
        self.clock_divider
    }

    pub fn get_prescaler_divider(&self) -> u8 {
        self.prescaler_divider
    }

    pub fn restart_sound(&mut self) {
        self.on = true;
    }

    pub fn set_counter_on(&mut self, value: bool) {
        self.counter_on = value;
    }

    pub fn get_counter_on(&self) -> bool {
        self.counter_on
    }

    pub fn set_counter(&mut self, value: u8) {
        assert!(value <= 64);
        assert!(value > 0);
        self.counter = (value - 1) * 4;
    }

    pub fn set_shift_register_width(&mut self, shift_register_width: bool) {
        self.shift_register_width = if shift_register_width {
            ShiftRegisterWidth::SevenBit
        } else {
            ShiftRegisterWidth::FifteenBit
        };
        // Reset shift register because we might get all zeroes if we switch in
        // the wrong moment
        self.shift_register = 0x7FFF;
    }

    pub fn get_shift_register_width(&self) -> bool {
        match self.shift_register_width {
            ShiftRegisterWidth::SevenBit => true,
            ShiftRegisterWidth::FifteenBit => false,
        }
    }

    pub fn set_envelope_starting_volume(&mut self, value: u8) {
        assert!(value < 0x10);
        self.envelope_starting_volume = value;
        self.volume = value;
    }

    pub fn get_envelope_starting_volume(&self) -> u8 {
        self.envelope_starting_volume
    }

    pub fn set_volume_envelope_direction(&mut self, direction: bool) {
        self.volume_envelope_direction = direction;
    }

    pub fn get_volume_envelope_direction(&self) -> bool {
        self.volume_envelope_direction
    }

    pub fn set_volume_envelope_period(&mut self, period: u8) {
        self.volume_envelope_period = period;
        self.volume_envelope_tick_counts = period;
    }

    pub fn get_volume_envelope_period(&self) -> u8 {
        self.volume_envelope_period
    }

    fn set_frequency(&mut self) {
        let mut frequency = 524_288;
        if self.clock_divider == 0 {
            frequency *= 2;
        } else {
            frequency /= self.clock_divider as u64;
        }
        frequency /= 1 << (self.prescaler_divider + 1);
        self.frequency_timer_ticks = Self::calc_num_timer_ticks_for_frequency(frequency);
    }

    fn calc_num_timer_ticks_for_frequency(frequency: u64) -> u32 {
        (SOUND_SAMPLE_RATE_IN_HERTZ / frequency) as u32 /
            FREQUENCY_TIMER_TICKS_PER_PERIOD
    }

    pub fn clock_tick(&mut self) {
        assert!(self.frequency_timer_tick_counts != 0);
        self.frequency_timer_tick_counts -= 1;
        if self.frequency_timer_tick_counts == 0 {
            self.frequency_timer_tick_counts = self.frequency_timer_ticks;
            self.shift_right();
        }
    }

    fn shift_right(&mut self) {
        let bit0 = self.shift_register & 0b01 != 0;
        let bit1 = self.shift_register & 0b10 != 0;
        let new_bit = bit0 != bit1;
        self.shift_register >>= 1;
        match self.shift_register_width {
            ShiftRegisterWidth::SevenBit => {
                self.shift_register &= 0b0011_1111_1011_1111;
                self.shift_register |= (new_bit as u16) << 6;
            }
            ShiftRegisterWidth::FifteenBit => {
                // should not be necessary
                self.shift_register &= 0b0011_1111_1111_1111;
            }
        }
        self.shift_register |= (new_bit as u16) << 14;
    }

    fn get_output(&self) -> i16 {
        if self.on {
            if self.shift_register & 0x1 != 0 { 1 } else { -1 }
        } else {
            0
        }
    }

    fn apply_volume_control(&self, sample: i16) -> i16 {
        sample * self.volume as i16 * (8000/15)
    }

    pub fn get_value(&self) -> i16 {
        self.apply_volume_control(self.get_output())
    }

    pub fn length_counter_tick(&mut self) {
        if self.counter_on {
            if self.counter > 0 {
                self.counter -= 1;
            }
            if self.counter == 0 {
                self.counter_on == false;
                self.on = false;
            }
        }
    }

    pub fn volume_envelope_tick(&mut self) {
        if !self.on || self.volume_envelope_period == 0 { return; }
        if self.volume_envelope_tick_counts > 0 {
            self.volume_envelope_tick_counts -= 1;
        }
        if self.volume_envelope_tick_counts == 0 {
            self.volume_envelope_tick_counts = self.volume_envelope_period;
            if self.volume > 0 && !self.volume_envelope_direction {
                self.volume -= 1;
            }
            if self.volume < 15 && self.volume_envelope_direction {
                self.volume += 1;
            }
        }
    }
}
