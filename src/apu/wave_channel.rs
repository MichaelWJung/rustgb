use super::SOUND_SAMPLE_RATE_IN_HERTZ;
const FREQUENCY_TIMER_TICKS_PER_PERIOD: u32 = 32;

pub struct WaveChannel {
    frequency_timer_ticks: u32,
    frequency_timer_tick_counts: u32,
    frequency_hi: u8,
    frequency_lo: u8,
    on: bool,
    counter_on: bool,
    counter: u16,
    volume: u8,
    pub wave_pattern: [u8; FREQUENCY_TIMER_TICKS_PER_PERIOD as usize],
    wave_pattern_step: u8,
}

impl WaveChannel {
    pub fn new() -> WaveChannel {
        // Default: 200 Hz
        let frequency_timer_ticks = Self::calc_num_timer_ticks_for_frequency(200);
        WaveChannel {
            frequency_timer_ticks,
            frequency_timer_tick_counts: frequency_timer_ticks,
            frequency_hi: 0,
            frequency_lo: 0,
            on: false,
            counter_on: false,
            counter: 0,
            volume: 3,
            wave_pattern: [0; FREQUENCY_TIMER_TICKS_PER_PERIOD as usize],
            wave_pattern_step: 0,
        }
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
        self.wave_pattern_step = 0;
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    pub fn get_on(&self) -> bool {
        self.on
    }

    pub fn set_counter_on(&mut self, value: bool) {
        self.counter_on = value;
    }

    pub fn get_counter_on(&self) -> bool {
        self.counter_on
    }

    pub fn set_counter(&mut self, value: u16) {
        assert!(value <= 256);
        assert!(value > 0);
        self.counter = (value - 1) * 4;
    }

    pub fn set_volume(&mut self, volume: u8) {
        assert!(volume < 4);
        self.volume = volume;
    }

    fn set_frequency(&mut self) {
        let val = ((self.frequency_hi as u16) << 8) + self.frequency_lo as u16;
        let frequency = 65536 / (2048 - val as u64);
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
            self.wave_pattern_step = (self.wave_pattern_step + 1) % FREQUENCY_TIMER_TICKS_PER_PERIOD as u8;
        }
    }

    fn get_output(&self) -> i16 {
        if self.on {
            self.wave_pattern[self.wave_pattern_step as usize] as i16
        } else {
            0
        }
    }

    fn apply_volume_control(&self, sample: i16) -> i16 {
        (sample >> (3 - self.volume)) * (8000/15)
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
}
