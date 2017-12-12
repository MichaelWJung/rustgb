use sdl2::Sdl;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

pub const OUTPUT_SAMPLE_RATE_IN_HERTZ: i32 = 48_000;

pub trait AudioDevice {
    fn queue(&self, bytes: &[i16]);
    fn queue_size(&self) -> usize;
}

struct SdlAudioDevice {
    audio_queue: AudioQueue<i16>,

}

impl SdlAudioDevice {
    fn new(audio_queue: AudioQueue<i16>) -> SdlAudioDevice {
        SdlAudioDevice {
            audio_queue
        }
    }
}

impl AudioDevice for SdlAudioDevice {
    fn queue(&self, bytes: &[i16]) {
        self.audio_queue.queue(bytes);
    }

    fn queue_size(&self) -> usize {
        self.audio_queue.size() as usize
    }
}

pub fn create_audio_device(sdl_context: &Sdl) -> Box<AudioDevice> {
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(OUTPUT_SAMPLE_RATE_IN_HERTZ),
        channels: Some(1), // mono
        samples: Some(512), // default sample size
    };
    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec).unwrap();
    device.resume();
    Box::new(SdlAudioDevice::new(device))
}

