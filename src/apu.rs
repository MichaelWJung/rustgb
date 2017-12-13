use audio::{AudioDevice, OUTPUT_SAMPLE_RATE_IN_HERTZ};
use cpu::CLOCK_SPEED_IN_HERTZ;
use std::collections::VecDeque;
//const CLOCK_TICKS_PER_SAMPLE: u64 = 8;
const FREQUENCY_TIMER_TICKS_PER_PERIOD: u32 = 8;
//const SOUND_SAMPLE_RATE_IN_HERTZ: u64 = CLOCK_SPEED_IN_HERTZ / CLOCK_TICKS_PER_SAMPLE;
const SOUND_SAMPLE_RATE_IN_HERTZ: u64 = OUTPUT_SAMPLE_RATE_IN_HERTZ as u64 * 8;
const CLOCK_TICKS_PER_SAMPLE: f64 = CLOCK_SPEED_IN_HERTZ as f64 / SOUND_SAMPLE_RATE_IN_HERTZ as f64;
const CLOCK_TICKS_PER_512_HERTZ: u64 = CLOCK_SPEED_IN_HERTZ / 512;
const TARGET_BUFFER_LENGTH_IN_SAMPLES: usize = 1000;
const TARGET_QUEUE_LENGTH_IN_SAMPLES: usize = 3000;
const BUFFER_PUSH_SIZE: usize = SOUND_SAMPLE_RATE_IN_HERTZ as usize /
    OUTPUT_SAMPLE_RATE_IN_HERTZ as usize *
    TARGET_BUFFER_LENGTH_IN_SAMPLES;
const PID_CONST_PROPORTIONAL_TERM: f64 = 0.0075;
const PID_CONST_INTEGRAL_TERM: f64 = 0.0001;

pub struct Channel1 {
    frequency_timer_ticks: u32,
    frequency_timer_tick_counts: u32,
    frequency_hi: u8,
    frequency_lo: u8,
    wave_step: u8,
    on: bool,
    counter_on: bool,
    counter: u8,
    volume: u8,
    envelope_starting_volume: u8,
    volume_envelope_direction: bool,
    volume_envelope_period: u8,
}

impl Channel1 {
    fn new() -> Channel1 {
        let frequency_timer_ticks = (SOUND_SAMPLE_RATE_IN_HERTZ / 200) as u32 /
            FREQUENCY_TIMER_TICKS_PER_PERIOD;
        Channel1 {
            frequency_timer_ticks,
            frequency_timer_tick_counts: frequency_timer_ticks,
            frequency_hi: 0,
            frequency_lo: 0,
            wave_step: 0,
            on: false,
            counter_on: false,
            counter: 0,
            volume: 16,
            envelope_starting_volume: 0,
            volume_envelope_direction: false,
            volume_envelope_period: 0,
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
        self.on = true;
    }

    pub fn set_counter_on(&mut self, value: bool) {
        self.counter_on = value;
        //println!("counter_on: {}", self.counter_on);
    }

    pub fn get_counter_on(&self) -> bool {
        self.counter_on
    }

    pub fn set_counter(&mut self, value: u8) {
        assert!(value < 64);
        self.counter = value;
        //println!("counter: {}", self.counter);
    }

    pub fn get_on(&self) -> bool {
        self.on
    }

    pub fn set_envelope_starting_volume(&mut self, value: u8) {
        assert!(value < 0x10);
        self.envelope_starting_volume = value;
        self.volume = value;
        //println!("envelope_starting_volume: {}", self.envelope_starting_volume);
    }

    pub fn get_envelope_starting_volume(&self) -> u8 {
        self.envelope_starting_volume
    }

    pub fn set_volume_envelope_direction(&mut self, direction: bool) {
        self.volume_envelope_direction = direction;
        //println!("volume_envelope_direction: {}", self.volume_envelope_direction);
    }

    pub fn get_volume_envelope_direction(&self) -> bool {
        self.volume_envelope_direction
    }

    pub fn set_volume_envelope_period(&mut self, period: u8) {
        self.volume_envelope_period = period;
        //println!("volume_envelope_period: {}", self.volume_envelope_period);
    }

    pub fn get_volume_envelope_period(&self) -> u8 {
        self.volume_envelope_period
    }

    fn set_frequency(&mut self) {
        let val = ((self.frequency_hi as u16) << 8) + self.frequency_lo as u16;
        let frequency = 131_072 / (2048 - val as u64);
        self.frequency_timer_ticks = (SOUND_SAMPLE_RATE_IN_HERTZ / frequency) as u32 /
            FREQUENCY_TIMER_TICKS_PER_PERIOD;
    }

    fn clock_tick(&mut self) {
        assert!(self.frequency_timer_tick_counts != 0);
        self.frequency_timer_tick_counts -= 1;
        if self.frequency_timer_tick_counts == 0 {
            self.frequency_timer_tick_counts = self.frequency_timer_ticks;
            self.wave_step = (self.wave_step + 1) % FREQUENCY_TIMER_TICKS_PER_PERIOD as u8;
        }
    }

    fn get_duty(&self) -> i16 {
        if self.on {
            if self.wave_step < 4 { 1 } else { -1 }
        } else {
            0
        }
    }

    fn apply_volume_control(&self, sample: i16) -> i16 {
        sample * self.volume as i16 * 8000/16
    }

    fn get_value(&self) -> i16 {
        self.apply_volume_control(self.get_duty())
    }

    fn length_counter_tick(&mut self) {
        if self.counter_on {
            if self.counter > 0 {
                self.counter -= 1;
            }
            if self.counter == 0 {
                self.on = false;
            }
        }
    }

    fn volume_envelope_tick(&mut self) {
        if self.volume_envelope_period != 0 {
            self.volume_envelope_period -= 1;
            if self.volume > 0 && !self.volume_envelope_direction {
                self.volume -= 1;
            }
            if self.volume < 15 && self.volume_envelope_direction {
                self.volume += 1;
            }
        }
    }
}

pub struct Apu<'a> {
    audio_device: &'a AudioDevice,
    buffer: Vec<i16>,
    sample_clock: f64,
    frame_sequencer_clock: u64,
    frame_sequencer_clock_counts: u8,
    first: bool,
    pitch: i16,
    queue_lengths: VecDeque<usize>,
    queue_diffence_integral: i32,
    sound_on: bool,
    pub channel1: Channel1,
}

impl<'a> Apu<'a> {
    pub fn new(audio_device: &AudioDevice) -> Apu {
        Apu {
            audio_device,
            buffer: Vec::new(),
            sample_clock: 0.0,
            frame_sequencer_clock: 0,
            frame_sequencer_clock_counts: 0,
            first: true,
            pitch: 0,
            queue_lengths: VecDeque::new(),
            queue_diffence_integral: 0,
            sound_on: false,
            channel1: Channel1::new(),
        }
    }

    pub fn step(&mut self, cycles_of_last_command: u8) {
        if self.first {
            let silence = vec![0; TARGET_QUEUE_LENGTH_IN_SAMPLES];
            self.audio_device.queue(&silence);
            self.first = false;
        }
        self.sample_clock += cycles_of_last_command as f64;
        while self.sample_clock > CLOCK_TICKS_PER_SAMPLE {
            self.sample_clock -= CLOCK_TICKS_PER_SAMPLE;
            self.clock_tick();
        }
        if self.sound_on {
            self.frame_sequencer_clock += cycles_of_last_command as u64;
            if self.frame_sequencer_clock >= CLOCK_TICKS_PER_512_HERTZ {
                self.frame_sequencer_clock -= CLOCK_TICKS_PER_512_HERTZ;
                self.frame_sequencer_tick();
            }
        }
    }

    fn clock_tick(&mut self) {
        self.channel1.clock_tick();
        let val = self.channel1.get_value();
        self.buffer.push(val);
        if self.buffer.len() >= BUFFER_PUSH_SIZE {
            self.resample_and_push();
        }
    }

    fn frame_sequencer_tick(&mut self) {
        self.frame_sequencer_clock_counts = (self.frame_sequencer_clock_counts + 1) % 8;
        if self.frame_sequencer_clock_counts % 2 == 0 {
            self.channel1.length_counter_tick();
        }
        if self.frame_sequencer_clock_counts == 7 {
            self.channel1.volume_envelope_tick();
        }
    }

    pub fn set_sound_on(&mut self, sound_on: bool) {
        self.sound_on = sound_on;
    }

    pub fn get_sound_on(&self) -> bool {
        self.sound_on
    }

    fn resample_and_push(&mut self) {
        self.manage_pitch();
        let resampled = self.resample();
        self.audio_device.queue(&resampled);
        self.buffer.clear();
    }

    fn manage_pitch(&mut self) {
        let queue_size = self.audio_device.queue_size();
        self.queue_lengths.push_back(queue_size);
        let len = self.queue_lengths.len();
        if len > 20 {
            self.queue_lengths.pop_front();
        }
        let average = self.queue_lengths.iter().sum::<usize>() / len;
        let difference = TARGET_QUEUE_LENGTH_IN_SAMPLES as i32 - average as i32;
        self.queue_diffence_integral += difference;
        self.pitch = (difference as f64 * PID_CONST_PROPORTIONAL_TERM +
                      self.queue_diffence_integral as f64 * PID_CONST_INTEGRAL_TERM) as i16;
    }

    fn resample(&self) -> Vec<i16> {
        let target_buffer_length = TARGET_BUFFER_LENGTH_IN_SAMPLES as i16 + self.pitch;
        let chunk_size = self.buffer.len() as f64 / target_buffer_length as f64;
        let mut resampled = Vec::new();
        for i in 0..target_buffer_length {
            let low = (i as f64 * chunk_size) as usize;
            let high = ((i + 1) as f64 * chunk_size) as usize;
            let average = self.buffer[low..high]
                .iter()
                .map(|x| *x as i32)
                .sum::<i32>() /
                (high - low) as i32;
            resampled.push(average as i16);
        }
        resampled
    }
}
