use gpu::Gpu;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};

pub trait Memory {
    fn read_byte(&self, address: u16) -> u8;
    fn read_word(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address);
        let high_byte = self.read_byte(address + 1);
        (high_byte as u16) << 8 + low_byte as u16
    }
    fn write_byte(&mut self, address: u16, value: u8);
    fn write_word(&mut self, address: u16, value: u16) {
        let low_byte = (value & 0xFF) as u8;
        let high_byte = ((value & 0xFF00) >> 8) as u8;
        self.write_byte(address, low_byte);
        self.write_byte(address + 1, high_byte);
    }
}

pub struct MemoryMap<'a, 'b: 'a> {
    bios_active: bool,
    bios: &'a mut BlockMemory,
    rom: BlockMemory,
    external_ram: BlockMemory,
    working_ram: BlockMemory,
    zero_page: BlockMemory,
    io: &'a RefCell<BlockMemory>,
    gpu: &'a RefCell<Gpu<'b>>,
}

impl<'a, 'b: 'a> MemoryMap<'a, 'b> {
    pub fn new(
        bios: &'a mut BlockMemory,
        gpu: &'a RefCell<Gpu<'b>>,
        rom: BlockMemory,
        io: &'a RefCell<BlockMemory>
        ) -> MemoryMap<'a, 'b> {
        MemoryMap {
            bios_active: true,
            bios,
            rom,
            external_ram: BlockMemory::new(0x4000),
            working_ram: BlockMemory::new(0x4000),
            zero_page: BlockMemory::new(0x4000),
            io,
            gpu
        }
    }

    fn address_to_type(&self, address: u16) -> (MemoryType, u16) {
        match address {
            0x0000 ... 0x00FF if self.bios_active => (MemoryType::Bios, address),
            0x0000 ... 0x7FFF => (MemoryType::Rom, address),
            0x8000 ... 0x9FFF => (MemoryType::GraphicsVram, address & 0x1FFF),
            0xA000 ... 0xBFFF => (MemoryType::ExternalRam, address & 0x1FFF),
            0xC000 ... 0xFDFF => (MemoryType::WorkingRam, address & 0x1FFF),
            0xFE00 ... 0xFEFF => (MemoryType::Sprites, address & 0xFF),
            0xFF00 ... 0xFF7F => (MemoryType::Io, 0), // FIXME: Implement
            0xFF80 ... 0xFFFF => (MemoryType::ZeroPage, address & 0x7F),
            _ => panic!("Memory address not known")
        }
    }
}

impl<'a, 'b: 'a> Memory for MemoryMap<'a, 'b> {
    fn read_byte(&self, address: u16) -> u8 {
        let (memory_type, address) = self.address_to_type(address);
        let gpu = self.gpu.borrow();
        let io = self.io.borrow();
        let memory = match memory_type {
            MemoryType::GraphicsVram => gpu.get_vram(),
            MemoryType::Bios => self.bios,
            MemoryType::Rom => &self.rom,
            MemoryType::ExternalRam => &self.external_ram,
            MemoryType::WorkingRam => &self.working_ram,
            MemoryType::Sprites => gpu.get_vram(),
            MemoryType::ZeroPage => &self.zero_page,
            MemoryType::Io => io.deref(),
        };
        memory.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        let (memory_type, address) = self.address_to_type(address);
        let mut gpu = self.gpu.borrow_mut();
        let mut io = self.io.borrow_mut();
        let memory = match memory_type {
            MemoryType::GraphicsVram => gpu.get_vram_mut(),
            MemoryType::Bios => self.bios,
            MemoryType::Rom => &mut self.rom,
            MemoryType::ExternalRam => &mut self.external_ram,
            MemoryType::WorkingRam => &mut self.working_ram,
            MemoryType::Sprites => gpu.get_sprites_mut(),
            MemoryType::ZeroPage => &mut self.zero_page,
            MemoryType::Io => io.deref_mut(),
        };
        memory.write_byte(address, value)
    }
}

enum MemoryType {
    Bios,
    Rom,
    GraphicsVram,
    ExternalRam,
    WorkingRam,
    Sprites,
    ZeroPage,
    Io,
}

pub struct BlockMemory {
    memory: Vec::<u8>,
}

impl BlockMemory {
    pub fn new(size: usize) -> BlockMemory {
        BlockMemory { memory: vec![0; size] }
    }

    pub fn new_from_file(file: &mut File) -> BlockMemory {
        let mut memory = Vec::<u8>::new();
        file.read_to_end(&mut memory).unwrap();
        BlockMemory { memory }
    }
}

impl Memory for BlockMemory {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}
