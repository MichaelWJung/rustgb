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

pub struct Apu<'a> {
    audio_device: &'a AudioDevice,
    buffer: Vec<i16>,
    sample_clock: f64,
    frame_sequencer_clock: u64,
    frame_sequencer_clock_counts: u8,
    frequency_timer_ticks: u32,
    frequency_timer_tick_counts: u32,
    wave_step: u8,
    first: bool,
    pitch: i16,
    queue_lengths: VecDeque<usize>,
    queue_diffence_integral: i32,
    frequency_hi: u8,
    frequency_lo: u8,
    sound_on: bool,
    channel1_on: bool,
    channel1_counter_on: bool,
    channel1_counter: u8,
}

impl<'a> Apu<'a> {
    pub fn new(audio_device: &AudioDevice) -> Apu {
        let frequency_timer_ticks = (SOUND_SAMPLE_RATE_IN_HERTZ / 200) as u32 /
            FREQUENCY_TIMER_TICKS_PER_PERIOD;
        Apu {
            audio_device,
            buffer: Vec::new(),
            sample_clock: 0.0,
            frame_sequencer_clock: 0,
            frame_sequencer_clock_counts: 0,
            frequency_timer_ticks,
            frequency_timer_tick_counts: frequency_timer_ticks,
            wave_step: 0,
            first: true,
            pitch: 0,
            queue_lengths: VecDeque::new(),
            queue_diffence_integral: 0,
            frequency_hi: 0,
            frequency_lo: 0,
            sound_on: false,
            channel1_on: false,
            channel1_counter_on: false,
            channel1_counter: 0,
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
        assert!(self.frequency_timer_tick_counts != 0);
        self.frequency_timer_tick_counts -= 1;
        if self.frequency_timer_tick_counts == 0 {
            self.frequency_timer_tick_counts = self.frequency_timer_ticks;
            self.wave_step = (self.wave_step + 1) % FREQUENCY_TIMER_TICKS_PER_PERIOD as u8;
        }
        let val = if self.channel1_on {
            if self.wave_step < 4 { 8000 } else { -8000 }
        } else {
            0
        };
        self.buffer.push(val);
        if self.buffer.len() >= BUFFER_PUSH_SIZE {
            self.resample_and_push();
        }
    }

    fn frame_sequencer_tick(&mut self) {
        self.frame_sequencer_clock_counts = (self.frame_sequencer_clock_counts + 1) % 8;
        if self.frame_sequencer_clock_counts % 2 == 0 {
            self.length_counter_tick();
        }
    }

    fn length_counter_tick(&mut self) {
        if self.channel1_counter_on {
            if self.channel1_counter > 0 {
                self.channel1_counter -= 1;
            }
            if self.channel1_counter == 0 {
                self.channel1_on = false;
            }
        }
    }

    pub fn set_frequency_hi(&mut self, frequency_hi: u8) {
        self.frequency_hi = frequency_hi;
        self.set_frequency();
    }

    pub fn set_frequency_lo(&mut self, frequency_lo: u8) {
        self.frequency_lo = frequency_lo;
    }

    pub fn get_frequency_hi(&self) -> u8 {
        self.frequency_hi
    }

    pub fn get_frequency_lo(&self) -> u8 {
        self.frequency_lo
    }

    pub fn set_sound_on(&mut self, sound_on: bool) {
        self.sound_on = sound_on;
    }

    pub fn get_sound_on(&self) -> bool {
        self.sound_on
    }

    pub fn restart_channel1_sound(&mut self) {
        self.channel1_on = true;
    }

    pub fn set_channel1_counter_on(&mut self, value: bool) {
        self.channel1_counter_on = value;
        //println!("channel1_counter_on: {}", self.channel1_counter_on);
    }

    pub fn get_channel1_counter_on(&self) -> bool {
        self.channel1_counter_on
    }

    pub fn set_channel1_counter(&mut self, value: u8) {
        assert!(value < 64);
        self.channel1_counter = value;
        //println!("channel1_counter: {}", self.channel1_counter);
    }

    pub fn get_channel1_on(&self) -> bool {
        self.channel1_on
    }

    fn set_frequency(&mut self) {
        let val = ((self.frequency_hi as u16) << 8) + self.frequency_lo as u16;
        let frequency = 131_072 / (2048 - val as u64);
        self.frequency_timer_ticks = (SOUND_SAMPLE_RATE_IN_HERTZ / frequency) as u32 /
            FREQUENCY_TIMER_TICKS_PER_PERIOD;
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
        //println!("average: {}", average);
        let difference = TARGET_QUEUE_LENGTH_IN_SAMPLES as i32 - average as i32;
        //println!("difference: {}", difference);
        self.queue_diffence_integral += difference;
        self.pitch = (difference as f64 * PID_CONST_PROPORTIONAL_TERM +
                      self.queue_diffence_integral as f64 * PID_CONST_INTEGRAL_TERM) as i16;
        //println!("pitch: {}", self.pitch);
        //println!("queue_size: {}", queue_size);
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

    //pub fn blub(&self) {
    //    let mut v = Vec::new();
    //    for _ in 0..256 {
    //        v.push(32767);
    //    }
    //    for _ in 0..256 {
    //        v.push(-32768);
    //    }
    //    self.audio_device.queue(&v);
    //}
}
