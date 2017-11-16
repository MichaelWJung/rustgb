extern crate sdl2;

mod cpu;
mod display;
mod gpu;
mod memory;

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::{thread, time};

const BIOS_PATH: &str = "roms/bios.gb";

pub fn run(file: &mut File) {
    let sdl_context = sdl2::init().unwrap();

    //let audio_device = audio::create_audio_device(&sdl_context);

    let mut display_context = display::DisplayContext::new(&sdl_context);
    let display = display::Display::new(&mut display_context);

    //let mut event_pump = sdl_context.event_pump().unwrap();
    //let keyboard = keyboard::Keyboard::new(&mut event_pump);

    //let mut memory = memory::BlockMemory::new();
    //memory.load_rom(file);

    let path = Path::new(BIOS_PATH);
    let mut bios = File::open(path).expect(&format!("Error opening file: {}", BIOS_PATH));

    let rom = memory::BlockMemory::new_from_file(file);
    let mut bios = memory::BlockMemory::new_from_file(&mut bios);
    let io = RefCell::new(memory::BlockMemory::new(0x80));
    let gpu = RefCell::new(gpu::Gpu::new(display, &io));
    let memory_map = memory::MemoryMap::new(&mut bios, &gpu, rom, &io);
    let mut cpu = cpu::Cpu::new(memory_map, &gpu);
    let mut next_frame = gpu::CLOCK_TICKS_PER_FRAME;
    let mut frame_start = time::Instant::now();
    loop {
        cpu.cycle();
        let clock = cpu.get_clock();
        if clock > next_frame {
            next_frame += gpu::CLOCK_TICKS_PER_FRAME;
            let duration = frame_start.elapsed();
            let frame_length_in_s = gpu::CLOCK_TICKS_PER_FRAME as f64 / cpu::CLOCK_SPEED_IN_HERTZ as f64;
            let frame_length_in_ns = (frame_length_in_s * 1e9) as u32;
            let frame_length = time::Duration::new(0, frame_length_in_ns);
            thread::sleep(duration - frame_length);
            frame_start = time::Instant::now();
        }
    }
}
