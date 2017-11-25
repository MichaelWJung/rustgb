use memory::{BlockMemory, Memory};

pub fn create_mbc(rom: BlockMemory) -> Box<Memory> {
    let mbc_type = rom.read_byte(0x147);
    println!("MBC type: {:#X}", mbc_type);
    match mbc_type {
        0x0 => Box::new(rom),
        0x1 | 0x2 | 0x3 => Box::new(Mbc1::new(rom)),
        0x19 ... 0x1E => Box::new(Mbc5::new(rom)),
        _ => panic!("Unsupported Memory Bank Controller {:#X}", mbc_type),
    }
}

struct Mbc1 {
    rom: BlockMemory,
    ram: BlockMemory,
    has_ram: bool,
    current_rom_bank: u8,
    current_ram_bank: u8,
    ram_enabled: bool,
    mode: RomRamMode,
    lower_bits: u8,
    upper_bits: u8,
}

impl Mbc1 {
    fn new(rom: BlockMemory) -> Mbc1 {
        let mbc_type = rom.read_byte(0x147);
        let has_ram = mbc_type == 0x2 || mbc_type == 0x3;
        Mbc1 {
            rom,
            ram: BlockMemory::new(0x8000),
            has_ram,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            mode: RomRamMode::RomBankingMode,
            lower_bits: 1,
            upper_bits: 0,
        }
    }

    fn update_bank_numbers(&mut self) {
        match self.mode {
            RomRamMode::RomBankingMode => {
                self.current_rom_bank = self.lower_bits + (self.upper_bits << 5);
                self.current_ram_bank = 0;
            }
            RomRamMode::RamBankingMode => {
                self.current_rom_bank = self.lower_bits;
                self.current_ram_bank = self.upper_bits;
            }
        }
    }
}

impl Memory for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.rom.read_byte(address),
            0x4000 ... 0x7FFF => {
                let address = (address & 0x3FFF) + self.current_rom_bank as u16 * 0x4000;
                self.rom.read_byte(address)
            }
            0xA000 ... 0xBFFF => {
                if self.ram_enabled {
                    let address = (address & 0x1FFF) + self.current_ram_bank as u16 * 0x2000;
                    self.ram.read_byte(address)
                } else {
                    0xFF
                }
            }
            _ => panic!("Mbc1 cannot handle read from address {:#X}.", address)
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x1FFF => self.ram_enabled = self.has_ram && (value & 0xF == 0xA),
            0x2000 ... 0x3FFF => {
                self.lower_bits = value & 0x1F;
                if self.lower_bits == 0x00 || self.lower_bits == 0x20 ||
                    self.lower_bits == 0x40 || self.lower_bits == 0x60 {
                    self.lower_bits += 1;
                }
                self.update_bank_numbers();
            }
            0x4000 ... 0x5FFF => {
                self.upper_bits = value & 0x3;
                self.update_bank_numbers();
            }
            0x6000 ... 0x7FFF => {
                if self.has_ram {
                    self.mode = if value & 0x1 != 0 {
                        RomRamMode::RomBankingMode
                    } else {
                        RomRamMode::RamBankingMode
                    };
                    self.update_bank_numbers();
                }
            }
            0xA000 ... 0xBFFF => {
                if self.ram_enabled {
                    let address = (address & 0x1FFF) + self.current_ram_bank as u16 * 0x2000;
                    self.ram.write_byte(address, value);
                }
            }
            _ => panic!("Mbc1 cannot handle write to address {:#X}.", address)
        }
    }
}

enum RomRamMode {
    RomBankingMode,
    RamBankingMode,
}

struct Mbc5 {
    rom: BlockMemory,
    ram: BlockMemory,
    has_ram: bool,
    current_rom_bank: u16,
    current_ram_bank: u8,
    ram_enabled: bool,
    lower_bits: u8,
    upper_bits: u8,
}

impl Mbc5 {
    fn new(rom: BlockMemory) -> Mbc5 {
        let mbc_type = rom.read_byte(0x147);
        let has_ram = mbc_type == 0x1A || mbc_type == 0x1B ||
                      mbc_type == 0x1D || mbc_type == 0x1E;
        Mbc5 {
            rom,
            ram: BlockMemory::new(0x20000),
            has_ram,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            lower_bits: 1,
            upper_bits: 0,
        }
    }

    fn update_rom_bank(&mut self) {
        self.current_rom_bank = ((self.upper_bits as u16) << 8) + self.lower_bits as u16;
    }
}

impl Memory for Mbc5 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.rom.read_byte(address),
            0x4000 ... 0x7FFF => {
                let address = (address & 0x3FFF) + self.current_rom_bank as u16 * 0x4000;
                self.rom.read_byte(address)
            }
            0xA000 ... 0xBFFF => {
                if self.ram_enabled {
                    let address = (address & 0x1FFF) + self.current_ram_bank as u16 * 0x2000;
                    self.ram.read_byte(address)
                } else {
                    0xFF
                }
            }
            _ => panic!("Mbc5 cannot handle read from address {:#X}.", address)
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x1FFF => self.ram_enabled = self.has_ram && (value & 0xF == 0xA),
            0x2000 ... 0x2FFF => {
                self.lower_bits = value;
                self.update_rom_bank();
            }
            0x3000 ... 0x3FFF => {
                self.upper_bits = value;
                self.update_rom_bank();
            }
            0x4000 ... 0x5FFF => {
                if self.has_ram {
                    self.current_ram_bank = value & 0xF;
                }
            }
            0xA000 ... 0xBFFF => {
                if self.ram_enabled {
                    let address = (address & 0x1FFF) + self.current_ram_bank as u16 * 0x2000;
                    self.ram.write_byte(address, value);
                }
            }
            //_ => panic!("Mbc5 cannot handle write to address {:#X}.", address)
            _ => (),
        }
    }
}
