extern crate app_dirs;
extern crate sdl2;

mod apu;
mod audio;
mod cpu;
mod display;
mod gpu;
mod io_registers;
mod keyboard;
mod mbc;
mod memory;
mod timer;

use memory::Memory;

use std::cell::RefCell;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use std::{thread, time};

const BIOS_PATH: &str = "roms/bios.gb";
const FRAME_LENGTH_IN_S: f64 = gpu::CLOCK_TICKS_PER_FRAME as f64 / cpu::CLOCK_SPEED_IN_HERTZ as f64;
const FRAME_LENGTH_IN_NS: u32 = (FRAME_LENGTH_IN_S * 1e9) as u32;

pub fn run(file: &mut File) {
    let sdl_context = sdl2::init().unwrap();

    let audio_device = audio::create_audio_device(&sdl_context);

    let display = display::SdlDisplay::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut keyboard = keyboard::Keyboard::new(&mut event_pump);

    let path = Path::new(BIOS_PATH);
    let mut bios = File::open(path).expect(&format!("Error opening file: {}", BIOS_PATH));

    let rom = memory::BlockMemory::new_from_file(file);
    let mbc = mbc::create_mbc(rom);
    let mut bios = memory::BlockMemory::new_from_file(&mut bios);
    let timer = RefCell::new(timer::Timer::new());
    let gpu = RefCell::new(gpu::Gpu::new(display));
    let apu = RefCell::new(apu::Apu::new(audio_device.deref()));
    let io = RefCell::new(io_registers::IoRegisters::new(&apu, &gpu, &timer));
    let memory_map = memory::MemoryMap::new(&mut bios, mbc, &gpu, &io);
    let mut cpu = cpu::Cpu::new(memory_map);

    let mut next_frame = gpu::CLOCK_TICKS_PER_FRAME as u64;
    let mut frame_start = time::Instant::now();
    let frame_length = time::Duration::new(0, FRAME_LENGTH_IN_NS);
    loop {
        let cycles_of_last_command = cpu.cycle();
        gpu.borrow_mut().step(cycles_of_last_command);
        timer.borrow_mut().increase(cycles_of_last_command);

        let mut key_register = io.borrow().read_byte(0x0);
        keyboard.update_key_register(&mut key_register);
        io.borrow_mut().write_byte(0x0, key_register);

        apu.borrow_mut().step(cycles_of_last_command);

        let clock = cpu.get_clock();
        if clock > next_frame {
            let pressed = keyboard.check_events();
            if pressed {
                let interrupts_fired = io.borrow().read_byte(0x0F);
                io.borrow_mut().write_byte(0x0F, interrupts_fired | 0b0001_0000);
            }
            next_frame += gpu::CLOCK_TICKS_PER_FRAME as u64;
            let duration = frame_start.elapsed();
            if frame_length > duration {
                thread::sleep(frame_length - duration);
            }
            frame_start = time::Instant::now();
        }

        if keyboard.program_end_triggered() {
            break;
        }
    }
}
