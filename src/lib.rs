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
const FRAME_LENGTH_IN_S: f64 = gpu::CLOCK_TICKS_PER_FRAME as f64 / cpu::CLOCK_SPEED_IN_HERTZ as f64;
const FRAME_LENGTH_IN_NS: u32 = (FRAME_LENGTH_IN_S * 1e9) as u32;

pub fn run(file: &mut File) {
    let sdl_context = sdl2::init().unwrap();

    //let audio_device = audio::create_audio_device(&sdl_context);

    let mut display_context = display::SdlDisplayContext::new(&sdl_context);
    let display = display::SdlDisplay::new(&mut display_context);

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
    let mut cpu = cpu::Cpu::new(memory_map);

    let mut next_frame = gpu::CLOCK_TICKS_PER_FRAME;
    let mut frame_start = time::Instant::now();
    let frame_length = time::Duration::new(0, FRAME_LENGTH_IN_NS);
    loop {
        let cycles_of_last_command = cpu.cycle();
        gpu.borrow_mut().step(cycles_of_last_command);
        let clock = cpu.get_clock();
        if clock > next_frame {
            next_frame += gpu::CLOCK_TICKS_PER_FRAME;
            let duration = frame_start.elapsed();
            if frame_length > duration {
                thread::sleep(frame_length - duration);
            }
            frame_start = time::Instant::now();
        }
    }
}
