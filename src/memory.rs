use std::fs::File;
use std::io::Read;

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
    //fn read_block(&self, address: u16, size: usize) -> &[u8];
}

pub struct MemoryMap {
    bios_active: bool,
    bios: BlockMemory,
    rom: BlockMemory,
    graphics_vram: BlockMemory,
    external_ram: BlockMemory,
    working_ram: BlockMemory,
    sprites: BlockMemory,
    zero_page: BlockMemory,
    io: BlockMemory,
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            bios_active: true,
            bios: BlockMemory::new(),
            rom: BlockMemory::new(),
            graphics_vram: BlockMemory::new(),
            external_ram: BlockMemory::new(),
            working_ram: BlockMemory::new(),
            sprites: BlockMemory::new(),
            zero_page: BlockMemory::new(),
            io: BlockMemory::new(),
        }
    }

    fn addressToType(&self, address: u16) -> (MemoryType, u16) {
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

impl Memory for MemoryMap {
    fn read_byte(&self, address: u16) -> u8 {
        let (memory_type, address) = self.addressToType(address);
        let memory = match memory_type {
            MemoryType::Bios => &self.bios,
            MemoryType::Rom => &self.rom,
            MemoryType::GraphicsVram => &self.graphics_vram,
            MemoryType::ExternalRam => &self.external_ram,
            MemoryType::WorkingRam => &self.working_ram,
            MemoryType::Sprites => &self.sprites,
            MemoryType::ZeroPage => &self.zero_page,
            MemoryType::Io => &self.io,
        };
        memory.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        let (memory_type, address) = self.addressToType(address);
        let memory = match memory_type {
            MemoryType::Bios => &mut self.bios,
            MemoryType::Rom => &mut self.rom,
            MemoryType::GraphicsVram => &mut self.graphics_vram,
            MemoryType::ExternalRam => &mut self.external_ram,
            MemoryType::WorkingRam => &mut self.working_ram,
            MemoryType::Sprites => &mut self.sprites,
            MemoryType::ZeroPage => &mut self.zero_page,
            MemoryType::Io => &mut self.io,
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
    memory: [u8; 4096],
}

impl BlockMemory {
    pub fn new() -> BlockMemory {
        let mut memory = BlockMemory { memory: [0; 4096] };
        //memory.initialize_sprites();
        memory
    }

    //pub fn load_rom(&mut self, file: &mut File) {
    //    let mut bytes = Vec::new();
    //    file.read_to_end(&mut bytes).unwrap();
    //    let size = bytes.len();
    //    for i in 0..size {
    //        self.memory[0x200 + i] = bytes[i];
    //    }
    //}

    //fn initialize_sprites(&mut self) {
    //    let numbers: [u8; 0x50] = [
    //        0xF0, 0x90, 0x90, 0x90, 0xF0,
    //        0x20, 0x60, 0x20, 0x20, 0x70,
    //        0xF0, 0x10, 0xF0, 0x80, 0xF0,
    //        0xF0, 0x10, 0xF0, 0x10, 0xF0,
    //        0x90, 0x90, 0xF0, 0x10, 0x10,
    //        0xF0, 0x80, 0xF0, 0x10, 0xF0,
    //        0xF0, 0x80, 0xF0, 0x90, 0xF0,
    //        0xF0, 0x10, 0x20, 0x40, 0x40,
    //        0xF0, 0x90, 0xF0, 0x90, 0xF0,
    //        0xF0, 0x90, 0xF0, 0x10, 0xF0,
    //        0xF0, 0x90, 0xF0, 0x90, 0x90,
    //        0xE0, 0x90, 0xE0, 0x90, 0xE0,
    //        0xF0, 0x80, 0x80, 0x80, 0xF0,
    //        0xE0, 0x90, 0x90, 0x90, 0xE0,
    //        0xF0, 0x80, 0xF0, 0x80, 0xF0,
    //        0xF0, 0x80, 0xF0, 0x80, 0x80,
    //    ];
    //    self.memory[..0x50].clone_from_slice(&numbers);
    //}
}

impl Memory for BlockMemory {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    //fn read_block(&self, address: u16, size: usize) -> &[u8] {
    //    let address = address as usize;
    //    &self.memory[address..(address + size)]
    //}
}
