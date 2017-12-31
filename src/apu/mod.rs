mod noise_channel;
mod square_channel;
mod volume_envelope;
mod wave_channel;
use self::noise_channel::NoiseChannel;
use self::square_channel::SquareChannel;
use self::wave_channel::WaveChannel;

use audio::{AudioDevice, OUTPUT_SAMPLE_RATE_IN_HERTZ};
use cpu::CLOCK_SPEED_IN_HERTZ;
use std::collections::VecDeque;
//const SOUND_SAMPLE_RATE_IN_HERTZ: u64 = OUTPUT_SAMPLE_RATE_IN_HERTZ as u64 * 16;
const SOUND_SAMPLE_RATE_IN_HERTZ: u64 = CLOCK_SPEED_IN_HERTZ / 4;
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
    first: bool,
    pitch: i16,
    queue_lengths: VecDeque<usize>,
    queue_diffence_integral: i32,
    sound_on: bool,
    pub channel1: SquareChannel,
    pub channel2: SquareChannel,
    pub channel3: WaveChannel,
    pub channel4: NoiseChannel,
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
            channel1: SquareChannel::new(),
            channel2: SquareChannel::new(),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
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
        self.channel2.clock_tick();
        self.channel3.clock_tick();
        self.channel4.clock_tick();
        let val1 = self.channel1.get_value();
        let val2 = self.channel2.get_value();
        let val3 = self.channel3.get_value();
        let val4 = self.channel4.get_value();
        self.buffer.push(val1 + val2 + val3 + val4);
        if self.buffer.len() >= BUFFER_PUSH_SIZE {
            self.resample_and_push();
        }
    }

    fn frame_sequencer_tick(&mut self) {
        self.frame_sequencer_clock_counts = (self.frame_sequencer_clock_counts + 1) % 8;
        if self.frame_sequencer_clock_counts % 2 == 0 {
            self.channel1.length_counter_tick();
            self.channel2.length_counter_tick();
            self.channel3.length_counter_tick();
            self.channel4.length_counter_tick();
        }
        if self.frame_sequencer_clock_counts == 7 {
            self.channel1.volume_envelope_tick();
            self.channel2.volume_envelope_tick();
            self.channel4.volume_envelope_tick();
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
