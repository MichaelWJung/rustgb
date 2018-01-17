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
    channel1: SquareChannel,
    channel2: SquareChannel,
    channel3: WaveChannel,
    channel4: NoiseChannel,
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

    // Sound channel 1: Square wave
    pub fn get_channel1_on(&self) -> bool {
        self.channel1.get_on()
    }

    pub fn restart_channel1_sound(&mut self) {
        self.channel1.restart_sound();
    }

    pub fn set_channel1_counter(&mut self, value: u8) {
        self.channel1.set_counter(value);
    }

    pub fn set_channel1_frequency_lo(&mut self, frequency_lo: u8) {
        self.channel1.set_frequency_lo(frequency_lo);
    }

    pub fn get_channel1_frequency_lo(&self) -> u8 {
        self.channel1.get_frequency_lo()
    }

    pub fn set_channel1_frequency_hi(&mut self, frequency_hi: u8) {
        self.channel1.set_frequency_hi(frequency_hi);
    }

    pub fn get_channel1_frequency_hi(&self) -> u8 {
        self.channel1.get_frequency_hi()
    }

    pub fn set_channel1_counter_on(&mut self, value: bool) {
        self.channel1.set_counter_on(value);
    }

    pub fn get_channel1_counter_on(&self) -> bool {
        self.channel1.get_counter_on()
    }

    pub fn set_channel1_duty(&mut self, duty: u8) {
        self.channel1.set_duty(duty);
    }

    pub fn get_channel1_duty(&self) -> u8 {
        self.channel1.get_duty()
    }

    pub fn set_channel1_envelope_starting_volume(&mut self, value: u8) {
        self.channel1.set_envelope_starting_volume(value);
    }

    pub fn get_channel1_envelope_starting_volume(&self) -> u8 {
        self.channel1.get_envelope_starting_volume()
    }

    pub fn set_channel1_volume_envelope_direction(&mut self, direction: bool) {
        self.channel1.set_volume_envelope_direction(direction);
    }

    pub fn get_channel1_volume_envelope_direction(&self) -> bool {
        self.channel1.get_volume_envelope_direction()
    }

    pub fn set_channel1_volume_envelope_period(&mut self, period: u8) {
        self.channel1.set_volume_envelope_period(period);
    }

    pub fn get_channel1_volume_envelope_period(&self) -> u8 {
        self.channel1.get_volume_envelope_period()
    }

    // Sound channel 2: Square wave
    pub fn get_channel2_on(&self) -> bool {
        self.channel2.get_on()
    }

    pub fn restart_channel2_sound(&mut self) {
        self.channel2.restart_sound();
    }

    pub fn set_channel2_counter(&mut self, value: u8) {
        self.channel2.set_counter(value);
    }

    pub fn set_channel2_frequency_lo(&mut self, frequency_lo: u8) {
        self.channel2.set_frequency_lo(frequency_lo);
    }

    pub fn get_channel2_frequency_lo(&self) -> u8 {
        self.channel2.get_frequency_lo()
    }

    pub fn set_channel2_frequency_hi(&mut self, frequency_hi: u8) {
        self.channel2.set_frequency_hi(frequency_hi);
    }

    pub fn get_channel2_frequency_hi(&self) -> u8 {
        self.channel2.get_frequency_hi()
    }

    pub fn set_channel2_counter_on(&mut self, value: bool) {
        self.channel2.set_counter_on(value);
    }

    pub fn get_channel2_counter_on(&self) -> bool {
        self.channel2.get_counter_on()
    }

    pub fn set_channel2_duty(&mut self, duty: u8) {
        self.channel2.set_duty(duty);
    }

    pub fn get_channel2_duty(&self) -> u8 {
        self.channel2.get_duty()
    }

    pub fn set_channel2_envelope_starting_volume(&mut self, value: u8) {
        self.channel2.set_envelope_starting_volume(value);
    }

    pub fn get_channel2_envelope_starting_volume(&self) -> u8 {
        self.channel2.get_envelope_starting_volume()
    }

    pub fn set_channel2_volume_envelope_direction(&mut self, direction: bool) {
        self.channel2.set_volume_envelope_direction(direction);
    }

    pub fn get_channel2_volume_envelope_direction(&self) -> bool {
        self.channel2.get_volume_envelope_direction()
    }

    pub fn set_channel2_volume_envelope_period(&mut self, period: u8) {
        self.channel2.set_volume_envelope_period(period);
    }

    pub fn get_channel2_volume_envelope_period(&self) -> u8 {
        self.channel2.get_volume_envelope_period()
    }

    // Sound channel 3: Wave channel
    pub fn set_channel3_on(&mut self, on: bool) {
        self.channel3.set_on(on)
    }

    pub fn get_channel3_on(&self) -> bool {
        self.channel3.get_on()
    }

    pub fn restart_channel3_sound(&mut self) {
        self.channel3.restart_sound();
    }

    pub fn set_channel3_counter(&mut self, value: u16) {
        self.channel3.set_counter(value);
    }

    pub fn set_channel3_volume(&mut self, volume: u8) {
        self.channel3.set_volume(volume);
    }

    pub fn set_channel3_frequency_lo(&mut self, frequency_lo: u8) {
        self.channel3.set_frequency_lo(frequency_lo);
    }

    pub fn get_channel3_frequency_lo(&self) -> u8 {
        self.channel3.get_frequency_lo()
    }

    pub fn set_channel3_frequency_hi(&mut self, frequency_hi: u8) {
        self.channel3.set_frequency_hi(frequency_hi);
    }

    pub fn get_channel3_frequency_hi(&self) -> u8 {
        self.channel3.get_frequency_hi()
    }

    pub fn set_channel3_counter_on(&mut self, value: bool) {
        self.channel3.set_counter_on(value);
    }

    pub fn get_channel3_counter_on(&self) -> bool {
        self.channel3.get_counter_on()
    }

    pub fn channel3_wave_pattern(&self) -> &[u8] {
        &self.channel3.wave_pattern
    }

    pub fn channel3_wave_pattern_mut(&mut self) -> &mut [u8] {
        &mut self.channel3.wave_pattern
    }

    // Sound channel 2: Noise
    pub fn restart_channel4_sound(&mut self) {
        self.channel4.restart_sound();
    }

    pub fn set_channel4_counter(&mut self, value: u8) {
        self.channel4.set_counter(value);
    }

    pub fn set_channel4_counter_on(&mut self, value: bool) {
        self.channel4.set_counter_on(value);
    }

    pub fn get_channel4_counter_on(&self) -> bool {
        self.channel4.get_counter_on()
    }

    pub fn set_channel4_envelope_starting_volume(&mut self, value: u8) {
        self.channel4.set_envelope_starting_volume(value);
    }

    pub fn get_channel4_envelope_starting_volume(&self) -> u8 {
        self.channel4.get_envelope_starting_volume()
    }

    pub fn set_channel4_volume_envelope_direction(&mut self, direction: bool) {
        self.channel4.set_volume_envelope_direction(direction);
    }

    pub fn get_channel4_volume_envelope_direction(&self) -> bool {
        self.channel4.get_volume_envelope_direction()
    }

    pub fn set_channel4_volume_envelope_period(&mut self, period: u8) {
        self.channel4.set_volume_envelope_period(period);
    }

    pub fn get_channel4_volume_envelope_period(&self) -> u8 {
        self.channel4.get_volume_envelope_period()
    }

    pub fn set_channel4_shift_register_width(&mut self, shift_register_width: bool) {
        self.channel4.set_shift_register_width(shift_register_width);
    }

    pub fn get_channel4_shift_register_width(&self) -> bool {
        self.channel4.get_shift_register_width()
    }

    pub fn set_channel4_clock_divider(&mut self, clock_divider: u8) {
        self.channel4.set_clock_divider(clock_divider);
    }

    pub fn get_channel4_clock_divider(&self) -> u8 {
        self.channel4.get_clock_divider()
    }

    pub fn set_channel4_prescaler_divider(&mut self, prescaler_divider: u8) {
        self.channel4.set_prescaler_divider(prescaler_divider);
    }

    pub fn get_channel4_prescaler_divider(&self) -> u8 {
        self.channel4.get_prescaler_divider()
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
