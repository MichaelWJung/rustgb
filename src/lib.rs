extern crate sdl2;

mod cpu;
mod memory;

use std::fs::File;
use std::{thread, time};

pub fn run(file: &mut File) {
    let sdl_context = sdl2::init().unwrap();

    //let audio_device = audio::create_audio_device(&sdl_context);

    //let mut display_context = display::DisplayContext::new(&sdl_context);
    //let display = display::Display::new(&mut display_context);

    //let mut event_pump = sdl_context.event_pump().unwrap();
    //let keyboard = keyboard::Keyboard::new(&mut event_pump);

    //let mut memory = memory::BlockMemory::new();
    //memory.load_rom(file);

    //let mut cpu = cpu::Cpu::new(memory);
    //loop {
    //    for _ in 0..10 {
    //        cpu.cycle();
    //    }
    //    thread::sleep(time::Duration::from_millis(17));
    //    cpu.decrement_timers();
    //    cpu.redraw_display();
    //}
}
