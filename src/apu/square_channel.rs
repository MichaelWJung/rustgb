use super::SOUND_SAMPLE_RATE_IN_HERTZ;
use super::FREQUENCY_TIMER_TICKS_PER_PERIOD;

pub struct SquareChannel {
    frequency_timer_ticks: u32,
    frequency_timer_tick_counts: u32,
    frequency_hi: u8,
    frequency_lo: u8,
    duty: u8,
    wave_step: u8,
    on: bool,
    counter_on: bool,
    counter: u8,
    volume: u8,
    envelope_starting_volume: u8,
    volume_envelope_direction: bool,
    volume_envelope_period: u8,
    volume_envelope_tick_counts: u8,
}

impl SquareChannel {
    pub fn new() -> SquareChannel {
        // Default: 200 Hz
        let frequency_timer_ticks = Self::calc_num_timer_ticks_for_frequency(200);
        SquareChannel {
            frequency_timer_ticks,
            frequency_timer_tick_counts: frequency_timer_ticks,
            frequency_hi: 0,
            frequency_lo: 0,
            duty: 2,
            wave_step: 0,
            on: false,
            counter_on: false,
            counter: 0,
            volume: 15,
            envelope_starting_volume: 0,
            volume_envelope_direction: false,
            volume_envelope_period: 0,
            volume_envelope_tick_counts: 0,
        }
    }

    pub fn set_duty(&mut self, duty: u8) {
        self.duty = duty;
    }

    pub fn get_duty(&self) -> u8 {
        self.duty
    }

    pub fn set_frequency_hi(&mut self, frequency_hi: u8) {
        self.frequency_hi = frequency_hi;
        self.set_frequency();
    }

    pub fn set_frequency_lo(&mut self, frequency_lo: u8) {
        self.frequency_lo = frequency_lo;
        self.set_frequency();
    }

    pub fn get_frequency_hi(&self) -> u8 {
        self.frequency_hi
    }

    pub fn get_frequency_lo(&self) -> u8 {
        self.frequency_lo
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

    pub fn get_on(&self) -> bool {
        self.on
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
        let val = ((self.frequency_hi as u16) << 8) + self.frequency_lo as u16;
        let frequency = 131_072 / (2048 - val as u64);
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
            self.wave_step = (self.wave_step + 1) % FREQUENCY_TIMER_TICKS_PER_PERIOD as u8;
        }
    }

    fn get_output(&self) -> i16 {
        if self.on {
            match self.duty {
                0 => if self.wave_step == 7 { 1 } else { -1 },
                1 => if self.wave_step == 0 || self.wave_step == 7 { 1 } else { -1 },
                2 => if self.wave_step < 4 { 1 } else { -1 },
                3 => if self.wave_step == 0 || self.wave_step == 7 { -1 } else { 1 },
                _ => panic!("Invalid duty cycle"),
            }
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
