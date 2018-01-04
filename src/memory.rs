use display::Display;
use io_registers::IoRegisters;
use gpu::Gpu;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;

pub trait Memory {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);

    fn read_word(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address);
        let high_byte = self.read_byte(address + 1);
        ((high_byte as u16) << 8) + low_byte as u16
    }

    fn read_4_bytes(&self, address: u16) -> [u8; 4] {
        [
            self.read_byte(address),
            self.read_byte(address + 1),
            self.read_byte(address + 2),
            self.read_byte(address + 3),
        ]
    }

    fn write_word(&mut self, address: u16, value: u16) {
        let low_byte = (value & 0xFF) as u8;
        let high_byte = ((value & 0xFF00) >> 8) as u8;
        self.write_byte(address, low_byte);
        self.write_byte(address + 1, high_byte);
    }

    // FIXME: fix this
    fn in_bios(&self) -> bool {
        false
    }

    fn leave_bios(&mut self) {}
}

pub struct MemoryMap<'a, 'b, D>
where
    'b: 'a,
    D: Display + 'b,
{
    bios_active: bool,
    bios: &'b mut BlockMemory,
    mbc: Box<Memory>,
    working_ram: BlockMemory,
    zero_page: BlockMemory,
    gpu: &'b RefCell<Gpu<D>>,
    io: &'a RefCell<IoRegisters<'b, D>>,
}

impl<'a, 'b, D> MemoryMap<'a, 'b, D>
where
    D: Display,
{
    pub fn new(
        bios: &'b mut BlockMemory,
        mbc: Box<Memory>,
        gpu: &'b RefCell<Gpu<D>>,
        io: &'a RefCell<IoRegisters<'b, D>>,
    ) -> MemoryMap<'a, 'b, D> {
        MemoryMap {
            bios_active: true,
            bios,
            mbc,
            working_ram: BlockMemory::new(0x2000),
            zero_page: BlockMemory::new(0x80),
            gpu,
            io,
        }
    }

    fn address_to_type(&self, address: u16) -> (MemoryType, u16) {
        match address {
            0x0000...0x00FF if self.bios_active => (MemoryType::Bios, address),
            0x0000...0x7FFF => (MemoryType::Mbc, address),
            0x8000...0x9FFF => (MemoryType::GraphicsVram, address & 0x1FFF),
            0xA000...0xBFFF => (MemoryType::Mbc, address),
            0xC000...0xFDFF => (MemoryType::WorkingRam, address & 0x1FFF),
            0xFE00...0xFEFF => (MemoryType::Sprites, address & 0xFF),
            0xFF00...0xFF7F => (MemoryType::Io, address & 0x7F),
            0xFF80...0xFFFF => (MemoryType::ZeroPage, address & 0x7F),
            _ => panic!("Memory address not known"),
        }
    }

    fn do_dma_transfer(&mut self, source: u8) {
        let start = (source as u16) << 8;
        for i in 0..0xA0 {
            let byte = self.read_byte(start + i);
            self.write_byte(0xFE00 + i, byte);
        }
    }
}

impl<'a, 'b, D> Memory for MemoryMap<'a, 'b, D>
where
    D: Display,
{
    fn read_byte(&self, address: u16) -> u8 {
        let (memory_type, address) = self.address_to_type(address);
        let gpu = self.gpu.borrow();
        let io = self.io.borrow();
        let memory = match memory_type {
            MemoryType::GraphicsVram => gpu.get_vram(),
            MemoryType::Bios => self.bios,
            MemoryType::Mbc => self.mbc.deref(),
            MemoryType::WorkingRam => &self.working_ram,
            MemoryType::Sprites => gpu.get_oam(),
            MemoryType::ZeroPage => &self.zero_page,
            MemoryType::Io => io.deref(),
        };
        memory.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        if address == 0xFF46 {
            self.do_dma_transfer(value);
        }
        let (memory_type, address) = self.address_to_type(address);
        match memory_type {
            MemoryType::GraphicsVram => {
                self.gpu.borrow_mut().get_vram_mut().write_byte(
                    address,
                    value,
                )
            }
            MemoryType::Bios => self.bios.write_byte(address, value),
            MemoryType::Mbc => self.mbc.write_byte(address, value),
            MemoryType::WorkingRam => self.working_ram.write_byte(address, value),
            MemoryType::Sprites => {
                self.gpu.borrow_mut().get_oam_mut().write_byte(
                    address,
                    value,
                )
            }
            MemoryType::ZeroPage => self.zero_page.write_byte(address, value),
            MemoryType::Io => self.io.borrow_mut().write_byte(address, value),
        };
    }

    fn in_bios(&self) -> bool {
        self.bios_active
    }

    fn leave_bios(&mut self) {
        self.bios_active = false;
    }
}

enum MemoryType {
    Bios,
    Mbc,
    GraphicsVram,
    WorkingRam,
    Sprites,
    ZeroPage,
    Io,
}

pub struct BlockMemory {
    memory: Vec<u8>,
    read_only: bool,
}

impl BlockMemory {
    pub fn new(size: usize) -> BlockMemory {
        BlockMemory {
            memory: vec![0; size],
            read_only: false,
        }
    }

    pub fn new_from_file(file: &mut File) -> BlockMemory {
        let mut memory = Vec::<u8>::new();
        file.read_to_end(&mut memory).unwrap();
        BlockMemory {
            memory,
            read_only: true,
        }
    }

    pub fn new_mut_from_file(file: &mut File) -> BlockMemory {
        let mut memory = Vec::<u8>::new();
        file.read_to_end(&mut memory).unwrap();
        BlockMemory {
            memory,
            read_only: false,
        }
    }

    pub fn read_byte_usize(&self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn write_byte_usize(&mut self, address: usize, value: u8) {
        if !self.read_only {
            self.memory[address] = value
        }
    }

    pub fn get_memory(&self) -> &Vec<u8> {
        &self.memory
    }
}

impl Memory for BlockMemory {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        if !self.read_only {
            self.memory[address as usize] = value;
        }
    }
}
