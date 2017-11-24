// Allow non Camel case types for opcode classes
#![allow(non_camel_case_types)]

use memory::Memory;

pub const CLOCK_SPEED_IN_HERTZ: u64 = 4_194_304;

pub struct Cpu<M>
{
    registers: Registers,
    memory: M,
    clock: u32,
}

impl<M> Cpu<M>
    where M: Memory
{
    pub fn new(memory: M) -> Cpu<M> {
        Cpu {
            registers: Registers::new(),
            memory,
            clock: 0,
        }
    }

    pub fn get_clock(&self) -> u32 {
        self.clock
    }

    pub fn cycle(&mut self) -> u8 {
        if self.memory.in_bios() && self.registers.pc == 0x100 {
            self.memory.leave_bios();
        }
        if self.registers.halt {
            self.registers.cycles_of_last_command = 4;
        } else {
            let opcode = self.fetch_opcode();
            self.execute_opcode(opcode);
        }
        self.clock += self.registers.cycles_of_last_command as u32;
        let mut cycles = self.registers.cycles_of_last_command;
        cycles += self.handle_interrupts();
        cycles
    }

    fn fetch_opcode(&self) -> Opcode {
        let pc = self.registers.pc;
        Opcode {
            b1: self.memory.read_byte(pc),
            b2: self.memory.read_byte(pc + 1),
            b3: self.memory.read_byte(pc + 2),
        }
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        let opcode_function = OPCODE_MAP[opcode.b1 as usize];
        opcode_function(opcode, &mut self.registers, &mut self.memory);
    }

    fn handle_interrupts(&mut self) -> u8 {
        let enabled_interrupts = self.memory.read_byte(0xFFFF);
        let interrupts_fired = self.memory.read_byte(0xFF0F) & enabled_interrupts;
        if interrupts_fired != 0 {
            self.registers.halt = false;
        }
        if !self.registers.interrupt_master_enable { return 0; }
        if interrupts_fired & Interrupt::VerticalBlank.to_bitmask() != 0 {
            self.start_interrupt_handler(Interrupt::VerticalBlank)
        } else if interrupts_fired & Interrupt::LcdStatus.to_bitmask() != 0 {
            self.start_interrupt_handler(Interrupt::LcdStatus)
        } else if interrupts_fired & Interrupt::Timer.to_bitmask() != 0 {
            self.start_interrupt_handler(Interrupt::Timer)
        } else if interrupts_fired & Interrupt::Serial.to_bitmask() != 0 {
            self.start_interrupt_handler(Interrupt::Serial)
        } else if interrupts_fired & Interrupt::Keypad.to_bitmask() != 0 {
            self.start_interrupt_handler(Interrupt::Keypad)
        } else {
            0
        }
    }

    fn start_interrupt_handler(&mut self, interrupt: Interrupt) -> u8 {
        self.registers.interrupt_master_enable = false;
        let interrupts_fired = self.memory.read_byte(0xFF0F);
        self.memory.write_byte(0xFF0F, interrupts_fired & !interrupt.to_bitmask());
        self.registers.sp -= 2;
        let sp = self.registers.sp;
        let pc = self.registers.pc;
        self.memory.write_word(sp, pc);
        self.registers.pc = interrupt.handler_address();
        12
    }
}

enum Interrupt {
    VerticalBlank,
    LcdStatus,
    Timer,
    Serial,
    Keypad,
}

impl Interrupt {
    fn to_bitmask(&self) -> u8 {
        match *self {
            Interrupt::VerticalBlank => 0b0000_0001,
            Interrupt::LcdStatus => 0b0000_0010,
            Interrupt::Timer => 0b0000_0100,
            Interrupt::Serial => 0b0000_1000,
            Interrupt::Keypad => 0b0001_0000,
        }
    }

    fn handler_address(&self) -> u16 {
        match *self {
            Interrupt::VerticalBlank => 0x40,
            Interrupt::LcdStatus => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Serial => 0x58,
            Interrupt::Keypad => 0x60,
        }
    }
}

static OPCODE_MAP: [fn(opcode: Opcode, registers: &mut Registers, memory: &mut Memory); 256] =
[
    create_and_execute::<NOP>,             // 0x00
    create_and_execute::<LD_BC_NN>,        // 0x01
    create_and_execute::<LD_xBC_A>,        // 0x02
    create_and_execute::<INC_BC>,          // 0x03
    create_and_execute::<INC_B>,           // 0x04
    create_and_execute::<DEC_B>,           // 0x05
    create_and_execute::<LD_B_N>,          // 0x06
    create_and_execute::<RLCA>,            // 0x07
    create_and_execute::<LD_xNN_SP>,       // 0x08
    create_and_execute::<ADD_HL_BC>,       // 0x09
    create_and_execute::<LD_A_xBC>,        // 0x0A
    create_and_execute::<DEC_BC>,          // 0x0B
    create_and_execute::<INC_C>,           // 0x0C
    create_and_execute::<DEC_C>,           // 0x0D
    create_and_execute::<LD_C_N>,          // 0x0E
    create_and_execute::<RRCA>,            // 0x0F
    create_and_execute::<STOP>,            // 0x10
    create_and_execute::<LD_DE_NN>,        // 0x11
    create_and_execute::<LD_xDE_A>,        // 0x12
    create_and_execute::<INC_DE>,          // 0x13
    create_and_execute::<INC_D>,           // 0x14
    create_and_execute::<DEC_D>,           // 0x15
    create_and_execute::<LD_D_N>,          // 0x16
    create_and_execute::<RLA>,             // 0x17
    create_and_execute::<JR_N>,            // 0x18
    create_and_execute::<ADD_HL_DE>,       // 0x19
    create_and_execute::<LD_A_xDE>,        // 0x1A
    create_and_execute::<DEC_DE>,          // 0x1B
    create_and_execute::<INC_E>,           // 0x1C
    create_and_execute::<DEC_E>,           // 0x1D
    create_and_execute::<LD_E_N>,          // 0x1E
    create_and_execute::<RRA>,             // 0x1F
    create_and_execute::<JR_NZ_N>,         // 0x20
    create_and_execute::<LD_HL_NN>,        // 0x21
    create_and_execute::<LDI_xHL_A>,       // 0x22
    create_and_execute::<INC_HL>,          // 0x23
    create_and_execute::<INC_H>,           // 0x24
    create_and_execute::<DEC_H>,           // 0x25
    create_and_execute::<LD_H_N>,          // 0x26
    create_and_execute::<DAA>,             // 0x27
    create_and_execute::<JR_Z_N>,          // 0x28
    create_and_execute::<ADD_HL_HL>,       // 0x29
    create_and_execute::<LDI_A_xHL>,       // 0x2A
    create_and_execute::<DEC_HL>,          // 0x2B
    create_and_execute::<INC_L>,           // 0x2C
    create_and_execute::<DEC_L>,           // 0x2D
    create_and_execute::<LD_L_N>,          // 0x2E
    create_and_execute::<CPL>,             // 0x2F
    create_and_execute::<JR_NC_N>,         // 0x30
    create_and_execute::<LD_SP_NN>,        // 0x31
    create_and_execute::<LDD_xHL_A>,       // 0x32
    create_and_execute::<INC_SP>,          // 0x33
    create_and_execute::<INC_xHL>,         // 0x34
    create_and_execute::<DEC_xHL>,         // 0x35
    create_and_execute::<LD_xHL_N>,        // 0x36
    create_and_execute::<SCF>,             // 0x37
    create_and_execute::<JR_C_N>,          // 0x38
    create_and_execute::<ADD_HL_SP>,       // 0x39
    create_and_execute::<LDD_A_xHL>,       // 0x3A
    create_and_execute::<DEC_SP>,          // 0x3B
    create_and_execute::<INC_A>,           // 0x3C
    create_and_execute::<DEC_A>,           // 0x3D
    create_and_execute::<LD_A_N>,          // 0x3E
    create_and_execute::<CCF>,             // 0x3F
    create_and_execute::<LD_B_B>,          // 0x40
    create_and_execute::<LD_B_C>,          // 0x41
    create_and_execute::<LD_B_D>,          // 0x42
    create_and_execute::<LD_B_E>,          // 0x43
    create_and_execute::<LD_B_H>,          // 0x44
    create_and_execute::<LD_B_L>,          // 0x45
    create_and_execute::<LD_B_xHL>,        // 0x46
    create_and_execute::<LD_B_A>,          // 0x47
    create_and_execute::<LD_C_B>,          // 0x48
    create_and_execute::<LD_C_C>,          // 0x49
    create_and_execute::<LD_C_D>,          // 0x4A
    create_and_execute::<LD_C_E>,          // 0x4B
    create_and_execute::<LD_C_H>,          // 0x4C
    create_and_execute::<LD_C_L>,          // 0x4D
    create_and_execute::<LD_C_xHL>,        // 0x4E
    create_and_execute::<LD_C_A>,          // 0x4F
    create_and_execute::<LD_D_B>,          // 0x50
    create_and_execute::<LD_D_C>,          // 0x51
    create_and_execute::<LD_D_D>,          // 0x52
    create_and_execute::<LD_D_E>,          // 0x53
    create_and_execute::<LD_D_H>,          // 0x54
    create_and_execute::<LD_D_L>,          // 0x55
    create_and_execute::<LD_D_xHL>,        // 0x56
    create_and_execute::<LD_D_A>,          // 0x57
    create_and_execute::<LD_E_B>,          // 0x58
    create_and_execute::<LD_E_C>,          // 0x59
    create_and_execute::<LD_E_D>,          // 0x5A
    create_and_execute::<LD_E_E>,          // 0x5B
    create_and_execute::<LD_E_H>,          // 0x5C
    create_and_execute::<LD_E_L>,          // 0x5D
    create_and_execute::<LD_E_xHL>,        // 0x5E
    create_and_execute::<LD_E_A>,          // 0x5F
    create_and_execute::<LD_H_B>,          // 0x60
    create_and_execute::<LD_H_C>,          // 0x61
    create_and_execute::<LD_H_D>,          // 0x62
    create_and_execute::<LD_H_E>,          // 0x63
    create_and_execute::<LD_H_H>,          // 0x64
    create_and_execute::<LD_H_L>,          // 0x65
    create_and_execute::<LD_H_xHL>,        // 0x66
    create_and_execute::<LD_H_A>,          // 0x67
    create_and_execute::<LD_L_B>,          // 0x68
    create_and_execute::<LD_L_C>,          // 0x69
    create_and_execute::<LD_L_D>,          // 0x6A
    create_and_execute::<LD_L_E>,          // 0x6B
    create_and_execute::<LD_L_H>,          // 0x6C
    create_and_execute::<LD_L_L>,          // 0x6D
    create_and_execute::<LD_L_xHL>,        // 0x6E
    create_and_execute::<LD_L_A>,          // 0x6F
    create_and_execute::<LD_xHL_B>,        // 0x70
    create_and_execute::<LD_xHL_C>,        // 0x71
    create_and_execute::<LD_xHL_D>,        // 0x72
    create_and_execute::<LD_xHL_E>,        // 0x73
    create_and_execute::<LD_xHL_H>,        // 0x74
    create_and_execute::<LD_xHL_L>,        // 0x75
    create_and_execute::<HALT>,            // 0x76
    create_and_execute::<LD_xHL_A>,        // 0x77
    create_and_execute::<LD_A_B>,          // 0x78
    create_and_execute::<LD_A_C>,          // 0x79
    create_and_execute::<LD_A_D>,          // 0x7A
    create_and_execute::<LD_A_E>,          // 0x7B
    create_and_execute::<LD_A_H>,          // 0x7C
    create_and_execute::<LD_A_L>,          // 0x7D
    create_and_execute::<LD_A_xHL>,        // 0x7E
    create_and_execute::<LD_A_A>,          // 0x7F
    create_and_execute::<ADD_A_B>,         // 0x80
    create_and_execute::<ADD_A_C>,         // 0x81
    create_and_execute::<ADD_A_D>,         // 0x82
    create_and_execute::<ADD_A_E>,         // 0x83
    create_and_execute::<ADD_A_H>,         // 0x84
    create_and_execute::<ADD_A_L>,         // 0x85
    create_and_execute::<ADD_A_xHL>,       // 0x86
    create_and_execute::<ADD_A_A>,         // 0x87
    create_and_execute::<ADC_A_B>,         // 0x88
    create_and_execute::<ADC_A_C>,         // 0x89
    create_and_execute::<ADC_A_D>,         // 0x8A
    create_and_execute::<ADC_A_E>,         // 0x8B
    create_and_execute::<ADC_A_H>,         // 0x8C
    create_and_execute::<ADC_A_L>,         // 0x8D
    create_and_execute::<ADC_A_xHL>,       // 0x8E
    create_and_execute::<ADC_A_A>,         // 0x8F
    create_and_execute::<SUB_A_B>,         // 0x90
    create_and_execute::<SUB_A_C>,         // 0x91
    create_and_execute::<SUB_A_D>,         // 0x92
    create_and_execute::<SUB_A_E>,         // 0x93
    create_and_execute::<SUB_A_H>,         // 0x94
    create_and_execute::<SUB_A_L>,         // 0x95
    create_and_execute::<SUB_A_xHL>,       // 0x96
    create_and_execute::<SUB_A_A>,         // 0x97
    create_and_execute::<SBC_A_B>,         // 0x98
    create_and_execute::<SBC_A_C>,         // 0x99
    create_and_execute::<SBC_A_D>,         // 0x9A
    create_and_execute::<SBC_A_E>,         // 0x9B
    create_and_execute::<SBC_A_H>,         // 0x9C
    create_and_execute::<SBC_A_L>,         // 0x9D
    create_and_execute::<SBC_A_xHL>,       // 0x9E
    create_and_execute::<SBC_A_A>,         // 0x9F
    create_and_execute::<AND_A_B>,         // 0xA0
    create_and_execute::<AND_A_C>,         // 0xA1
    create_and_execute::<AND_A_D>,         // 0xA2
    create_and_execute::<AND_A_E>,         // 0xA3
    create_and_execute::<AND_A_H>,         // 0xA4
    create_and_execute::<AND_A_L>,         // 0xA5
    create_and_execute::<AND_A_xHL>,       // 0xA6
    create_and_execute::<AND_A_A>,         // 0xA7
    create_and_execute::<XOR_A_B>,         // 0xA8
    create_and_execute::<XOR_A_C>,         // 0xA9
    create_and_execute::<XOR_A_D>,         // 0xAA
    create_and_execute::<XOR_A_E>,         // 0xAB
    create_and_execute::<XOR_A_H>,         // 0xAC
    create_and_execute::<XOR_A_L>,         // 0xAD
    create_and_execute::<XOR_A_xHL>,       // 0xAE
    create_and_execute::<XOR_A_A>,         // 0xAF
    create_and_execute::<OR_A_B>,          // 0xB0
    create_and_execute::<OR_A_C>,          // 0xB1
    create_and_execute::<OR_A_D>,          // 0xB2
    create_and_execute::<OR_A_E>,          // 0xB3
    create_and_execute::<OR_A_H>,          // 0xB4
    create_and_execute::<OR_A_L>,          // 0xB5
    create_and_execute::<OR_A_xHL>,        // 0xB6
    create_and_execute::<OR_A_A>,          // 0xB7
    create_and_execute::<CP_B>,            // 0xB8
    create_and_execute::<CP_C>,            // 0xB9
    create_and_execute::<CP_D>,            // 0xBA
    create_and_execute::<CP_E>,            // 0xBB
    create_and_execute::<CP_H>,            // 0xBC
    create_and_execute::<CP_L>,            // 0xBD
    create_and_execute::<CP_xHL>,          // 0xBE
    create_and_execute::<CP_A>,            // 0xBF
    create_and_execute::<RET_NZ>,          // 0xC0
    create_and_execute::<POP_BC>,          // 0xC1
    create_and_execute::<JP_NZ_NN>,        // 0xC2
    create_and_execute::<JP_NN>,           // 0xC3
    create_and_execute::<CALL_NZ_NN>,      // 0xC4
    create_and_execute::<PUSH_BC>,         // 0xC5
    create_and_execute::<ADD_A_N>,         // 0xC6
    create_and_execute::<RST_0x00>,        // 0xC7
    create_and_execute::<RET_Z>,           // 0xC8
    create_and_execute::<RET>,             // 0xC9
    create_and_execute::<JP_Z_NN>,         // 0xCA
    execute_extended_opcode,               // 0xCB
    create_and_execute::<CALL_Z_NN>,       // 0xCC
    create_and_execute::<CALL_NN>,         // 0xCD
    create_and_execute::<ADC_A_N>,         // 0xCE
    create_and_execute::<RST_0x08>,        // 0xCF
    create_and_execute::<RET_NC>,          // 0xD0
    create_and_execute::<POP_DE>,          // 0xD1
    create_and_execute::<JP_NC_NN>,        // 0xD2
    create_and_execute::<XX>,              // 0xD3
    create_and_execute::<CALL_NC_NN>,      // 0xD4
    create_and_execute::<PUSH_DE>,         // 0xD5
    create_and_execute::<SUB_A_N>,         // 0xD6
    create_and_execute::<RST_0x10>,        // 0xD7
    create_and_execute::<RET_C>,           // 0xD8
    create_and_execute::<RETI>,            // 0xD9
    create_and_execute::<JP_C_NN>,         // 0xDA
    create_and_execute::<XX>,              // 0xDB
    create_and_execute::<CALL_C_NN>,       // 0xDC
    create_and_execute::<XX>,              // 0xDD
    create_and_execute::<SBC_A_N>,         // 0xDE
    create_and_execute::<RST_0x18>,        // 0xDF
    create_and_execute::<LDH_xN_A>,        // 0xE0
    create_and_execute::<POP_HL>,          // 0xE1
    create_and_execute::<LDH_xC_A>,        // 0xE2
    create_and_execute::<XX>,              // 0xE3
    create_and_execute::<XX>,              // 0xE4
    create_and_execute::<PUSH_HL>,         // 0xE5
    create_and_execute::<AND_A_N>,         // 0xE6
    create_and_execute::<RST_0x20>,        // 0xE7
    create_and_execute::<ADD_SP_N>,        // 0xE8
    create_and_execute::<JP_xHL>,          // 0xE9
    create_and_execute::<LD_xNN_A>,        // 0xEA
    create_and_execute::<XX>,              // 0xEB
    create_and_execute::<XX>,              // 0xEC
    create_and_execute::<XX>,              // 0xED
    create_and_execute::<XOR_A_N>,         // 0xEE
    create_and_execute::<RST_0x28>,        // 0xEF
    create_and_execute::<LDH_A_xN>,        // 0xF0
    create_and_execute::<POP_AF>,          // 0xF1
    create_and_execute::<LDH_A_xC>,        // 0xF2  removed???
    create_and_execute::<DI>,              // 0xF3
    create_and_execute::<XX>,              // 0xE4
    create_and_execute::<PUSH_AF>,         // 0xF5
    create_and_execute::<OR_A_N>,          // 0xF6
    create_and_execute::<RST_0x30>,        // 0xF7
    create_and_execute::<LDHL_SP_N>,       // 0xF8
    create_and_execute::<LD_SP_HL>,        // 0xF9
    create_and_execute::<LD_A_xNN>,        // 0xFA
    create_and_execute::<EI>,              // 0xFB
    create_and_execute::<XX>,              // 0xFC
    create_and_execute::<XX>,              // 0xFD
    create_and_execute::<CP_N>,            // 0xFE
    create_and_execute::<RST_0x38>,        // 0xFF
];

static EXTENDED_OPCODE_MAP: [fn(opcode: Opcode, registers: &mut Registers, memory: &mut Memory); 256] =
[
    create_and_execute::<RLC_B>,     // 0x00
    create_and_execute::<RLC_C>,     // 0x01
    create_and_execute::<RLC_D>,     // 0x02
    create_and_execute::<RLC_E>,     // 0x03
    create_and_execute::<RLC_H>,     // 0x04
    create_and_execute::<RLC_L>,     // 0x05
    create_and_execute::<RLC_xHL>,   // 0x06
    create_and_execute::<RLC_A>,     // 0x07
    create_and_execute::<RRC_B>,     // 0x08
    create_and_execute::<RRC_C>,     // 0x09
    create_and_execute::<RRC_D>,     // 0x0A
    create_and_execute::<RRC_E>,     // 0x0B
    create_and_execute::<RRC_H>,     // 0x0C
    create_and_execute::<RRC_L>,     // 0x0D
    create_and_execute::<RRC_xHL>,   // 0x0E
    create_and_execute::<RRC_A>,     // 0x0F
    create_and_execute::<RL_B>,      // 0x10
    create_and_execute::<RL_C>,      // 0x11
    create_and_execute::<RL_D>,      // 0x12
    create_and_execute::<RL_E>,      // 0x13
    create_and_execute::<RL_H>,      // 0x14
    create_and_execute::<RL_L>,      // 0x15
    create_and_execute::<RL_xHL>,    // 0x16
    create_and_execute::<RL_A>,      // 0x17
    create_and_execute::<RR_B>,      // 0x18
    create_and_execute::<RR_C>,      // 0x19
    create_and_execute::<RR_D>,      // 0x1A
    create_and_execute::<RR_E>,      // 0x1B
    create_and_execute::<RR_H>,      // 0x1C
    create_and_execute::<RR_L>,      // 0x1D
    create_and_execute::<RR_xHL>,    // 0x1E
    create_and_execute::<RR_A>,      // 0x1F
    create_and_execute::<SLA_B>,     // 0x20
    create_and_execute::<SLA_C>,     // 0x21
    create_and_execute::<SLA_D>,     // 0x22
    create_and_execute::<SLA_E>,     // 0x23
    create_and_execute::<SLA_H>,     // 0x24
    create_and_execute::<SLA_L>,     // 0x25
    create_and_execute::<SLA_xHL>,   // 0x26
    create_and_execute::<SLA_A>,     // 0x27
    create_and_execute::<SRA_B>,     // 0x28
    create_and_execute::<SRA_C>,     // 0x29
    create_and_execute::<SRA_D>,     // 0x2A
    create_and_execute::<SRA_E>,     // 0x2B
    create_and_execute::<SRA_H>,     // 0x2C
    create_and_execute::<SRA_L>,     // 0x2D
    create_and_execute::<SRA_xHL>,   // 0x2E
    create_and_execute::<SRA_A>,     // 0x2F
    create_and_execute::<SWAP_B>,    // 0x30
    create_and_execute::<SWAP_C>,    // 0x31
    create_and_execute::<SWAP_D>,    // 0x32
    create_and_execute::<SWAP_E>,    // 0x33
    create_and_execute::<SWAP_H>,    // 0x34
    create_and_execute::<SWAP_L>,    // 0x35
    create_and_execute::<SWAP_xHL>,  // 0x36
    create_and_execute::<SWAP_A>,    // 0x37
    create_and_execute::<SRL_B>,     // 0x38
    create_and_execute::<SRL_C>,     // 0x39
    create_and_execute::<SRL_D>,     // 0x3A
    create_and_execute::<SRL_E>,     // 0x3B
    create_and_execute::<SRL_H>,     // 0x3C
    create_and_execute::<SRL_L>,     // 0x3D
    create_and_execute::<SRL_xHL>,   // 0x3E
    create_and_execute::<SRL_A>,     // 0x3F
    create_and_execute::<BIT_0_B>,   // 0x40
    create_and_execute::<BIT_0_C>,   // 0x41
    create_and_execute::<BIT_0_D>,   // 0x42
    create_and_execute::<BIT_0_E>,   // 0x43
    create_and_execute::<BIT_0_H>,   // 0x44
    create_and_execute::<BIT_0_L>,   // 0x45
    create_and_execute::<BIT_0_xHL>, // 0x46
    create_and_execute::<BIT_0_A>,   // 0x47
    create_and_execute::<BIT_1_B>,   // 0x48
    create_and_execute::<BIT_1_C>,   // 0x49
    create_and_execute::<BIT_1_D>,   // 0x4A
    create_and_execute::<BIT_1_E>,   // 0x4B
    create_and_execute::<BIT_1_H>,   // 0x4C
    create_and_execute::<BIT_1_L>,   // 0x4D
    create_and_execute::<BIT_1_xHL>, // 0x4E
    create_and_execute::<BIT_1_A>,   // 0x4F
    create_and_execute::<BIT_2_B>,   // 0x50
    create_and_execute::<BIT_2_C>,   // 0x51
    create_and_execute::<BIT_2_D>,   // 0x52
    create_and_execute::<BIT_2_E>,   // 0x53
    create_and_execute::<BIT_2_H>,   // 0x54
    create_and_execute::<BIT_2_L>,   // 0x55
    create_and_execute::<BIT_2_xHL>, // 0x56
    create_and_execute::<BIT_2_A>,   // 0x57
    create_and_execute::<BIT_3_B>,   // 0x58
    create_and_execute::<BIT_3_C>,   // 0x59
    create_and_execute::<BIT_3_D>,   // 0x5A
    create_and_execute::<BIT_3_E>,   // 0x5B
    create_and_execute::<BIT_3_H>,   // 0x5C
    create_and_execute::<BIT_3_L>,   // 0x5D
    create_and_execute::<BIT_3_xHL>, // 0x5E
    create_and_execute::<BIT_3_A>,   // 0x5F
    create_and_execute::<BIT_4_B>,   // 0x60
    create_and_execute::<BIT_4_C>,   // 0x61
    create_and_execute::<BIT_4_D>,   // 0x62
    create_and_execute::<BIT_4_E>,   // 0x63
    create_and_execute::<BIT_4_H>,   // 0x64
    create_and_execute::<BIT_4_L>,   // 0x65
    create_and_execute::<BIT_4_xHL>, // 0x66
    create_and_execute::<BIT_4_A>,   // 0x67
    create_and_execute::<BIT_5_B>,   // 0x68
    create_and_execute::<BIT_5_C>,   // 0x69
    create_and_execute::<BIT_5_D>,   // 0x6A
    create_and_execute::<BIT_5_E>,   // 0x6B
    create_and_execute::<BIT_5_H>,   // 0x6C
    create_and_execute::<BIT_5_L>,   // 0x6D
    create_and_execute::<BIT_5_xHL>, // 0x6E
    create_and_execute::<BIT_5_A>,   // 0x6F
    create_and_execute::<BIT_6_B>,   // 0x70
    create_and_execute::<BIT_6_C>,   // 0x71
    create_and_execute::<BIT_6_D>,   // 0x72
    create_and_execute::<BIT_6_E>,   // 0x73
    create_and_execute::<BIT_6_H>,   // 0x74
    create_and_execute::<BIT_6_L>,   // 0x75
    create_and_execute::<BIT_6_xHL>, // 0x76
    create_and_execute::<BIT_6_A>,   // 0x77
    create_and_execute::<BIT_7_B>,   // 0x78
    create_and_execute::<BIT_7_C>,   // 0x79
    create_and_execute::<BIT_7_D>,   // 0x7A
    create_and_execute::<BIT_7_E>,   // 0x7B
    create_and_execute::<BIT_7_H>,   // 0x7C
    create_and_execute::<BIT_7_L>,   // 0x7D
    create_and_execute::<BIT_7_xHL>, // 0x7E
    create_and_execute::<BIT_7_A>,   // 0x7F
    create_and_execute::<RES_0_B>,   // 0x80
    create_and_execute::<RES_0_C>,   // 0x81
    create_and_execute::<RES_0_D>,   // 0x82
    create_and_execute::<RES_0_E>,   // 0x83
    create_and_execute::<RES_0_H>,   // 0x84
    create_and_execute::<RES_0_L>,   // 0x85
    create_and_execute::<RES_0_xHL>, // 0x86
    create_and_execute::<RES_0_A>,   // 0x87
    create_and_execute::<RES_1_B>,   // 0x88
    create_and_execute::<RES_1_C>,   // 0x89
    create_and_execute::<RES_1_D>,   // 0x8A
    create_and_execute::<RES_1_E>,   // 0x8B
    create_and_execute::<RES_1_H>,   // 0x8C
    create_and_execute::<RES_1_L>,   // 0x8D
    create_and_execute::<RES_1_xHL>, // 0x8E
    create_and_execute::<RES_1_A>,   // 0x8F
    create_and_execute::<RES_2_B>,   // 0x90
    create_and_execute::<RES_2_C>,   // 0x91
    create_and_execute::<RES_2_D>,   // 0x92
    create_and_execute::<RES_2_E>,   // 0x93
    create_and_execute::<RES_2_H>,   // 0x94
    create_and_execute::<RES_2_L>,   // 0x95
    create_and_execute::<RES_2_xHL>, // 0x96
    create_and_execute::<RES_2_A>,   // 0x97
    create_and_execute::<RES_3_B>,   // 0x98
    create_and_execute::<RES_3_C>,   // 0x99
    create_and_execute::<RES_3_D>,   // 0x9A
    create_and_execute::<RES_3_E>,   // 0x9B
    create_and_execute::<RES_3_H>,   // 0x9C
    create_and_execute::<RES_3_L>,   // 0x9D
    create_and_execute::<RES_3_xHL>, // 0x9E
    create_and_execute::<RES_3_A>,   // 0x9F
    create_and_execute::<RES_4_B>,   // 0xA0
    create_and_execute::<RES_4_C>,   // 0xA1
    create_and_execute::<RES_4_D>,   // 0xA2
    create_and_execute::<RES_4_E>,   // 0xA3
    create_and_execute::<RES_4_H>,   // 0xA4
    create_and_execute::<RES_4_L>,   // 0xA5
    create_and_execute::<RES_4_xHL>, // 0xA6
    create_and_execute::<RES_4_A>,   // 0xA7
    create_and_execute::<RES_5_B>,   // 0xA8
    create_and_execute::<RES_5_C>,   // 0xA9
    create_and_execute::<RES_5_D>,   // 0xAA
    create_and_execute::<RES_5_E>,   // 0xAB
    create_and_execute::<RES_5_H>,   // 0xAC
    create_and_execute::<RES_5_L>,   // 0xAD
    create_and_execute::<RES_5_xHL>, // 0xAE
    create_and_execute::<RES_5_A>,   // 0xAF
    create_and_execute::<RES_6_B>,   // 0xB0
    create_and_execute::<RES_6_C>,   // 0xB1
    create_and_execute::<RES_6_D>,   // 0xB2
    create_and_execute::<RES_6_E>,   // 0xB3
    create_and_execute::<RES_6_H>,   // 0xB4
    create_and_execute::<RES_6_L>,   // 0xB5
    create_and_execute::<RES_6_xHL>, // 0xB6
    create_and_execute::<RES_6_A>,   // 0xB7
    create_and_execute::<RES_7_B>,   // 0xB8
    create_and_execute::<RES_7_C>,   // 0xB9
    create_and_execute::<RES_7_D>,   // 0xBA
    create_and_execute::<RES_7_E>,   // 0xBB
    create_and_execute::<RES_7_H>,   // 0xBC
    create_and_execute::<RES_7_L>,   // 0xBD
    create_and_execute::<RES_7_xHL>, // 0xBE
    create_and_execute::<RES_7_A>,   // 0xBF
    create_and_execute::<SET_0_B>,   // 0xC0
    create_and_execute::<SET_0_C>,   // 0xC1
    create_and_execute::<SET_0_D>,   // 0xC2
    create_and_execute::<SET_0_E>,   // 0xC3
    create_and_execute::<SET_0_H>,   // 0xC4
    create_and_execute::<SET_0_L>,   // 0xC5
    create_and_execute::<SET_0_xHL>, // 0xC6
    create_and_execute::<SET_0_A>,   // 0xC7
    create_and_execute::<SET_1_B>,   // 0xC8
    create_and_execute::<SET_1_C>,   // 0xC9
    create_and_execute::<SET_1_D>,   // 0xCA
    create_and_execute::<SET_1_E>,   // 0xCB
    create_and_execute::<SET_1_H>,   // 0xCC
    create_and_execute::<SET_1_L>,   // 0xCD
    create_and_execute::<SET_1_xHL>, // 0xCE
    create_and_execute::<SET_1_A>,   // 0xCF
    create_and_execute::<SET_2_B>,   // 0xD0
    create_and_execute::<SET_2_C>,   // 0xD1
    create_and_execute::<SET_2_D>,   // 0xD2
    create_and_execute::<SET_2_E>,   // 0xD3
    create_and_execute::<SET_2_H>,   // 0xD4
    create_and_execute::<SET_2_L>,   // 0xD5
    create_and_execute::<SET_2_xHL>, // 0xD6
    create_and_execute::<SET_2_A>,   // 0xD7
    create_and_execute::<SET_3_B>,   // 0xD8
    create_and_execute::<SET_3_C>,   // 0xD9
    create_and_execute::<SET_3_D>,   // 0xDA
    create_and_execute::<SET_3_E>,   // 0xDB
    create_and_execute::<SET_3_H>,   // 0xDC
    create_and_execute::<SET_3_L>,   // 0xDD
    create_and_execute::<SET_3_xHL>, // 0xDE
    create_and_execute::<SET_3_A>,   // 0xDF
    create_and_execute::<SET_4_B>,   // 0xE0
    create_and_execute::<SET_4_C>,   // 0xE1
    create_and_execute::<SET_4_D>,   // 0xE2
    create_and_execute::<SET_4_E>,   // 0xE3
    create_and_execute::<SET_4_H>,   // 0xE4
    create_and_execute::<SET_4_L>,   // 0xE5
    create_and_execute::<SET_4_xHL>, // 0xE6
    create_and_execute::<SET_4_A>,   // 0xE7
    create_and_execute::<SET_5_B>,   // 0xE8
    create_and_execute::<SET_5_C>,   // 0xE9
    create_and_execute::<SET_5_D>,   // 0xEA
    create_and_execute::<SET_5_E>,   // 0xEB
    create_and_execute::<SET_5_H>,   // 0xEC
    create_and_execute::<SET_5_L>,   // 0xED
    create_and_execute::<SET_5_xHL>, // 0xEE
    create_and_execute::<SET_5_A>,   // 0xEF
    create_and_execute::<SET_6_B>,   // 0xF0
    create_and_execute::<SET_6_C>,   // 0xF1
    create_and_execute::<SET_6_D>,   // 0xF2
    create_and_execute::<SET_6_E>,   // 0xF3
    create_and_execute::<SET_6_H>,   // 0xF4
    create_and_execute::<SET_6_L>,   // 0xF5
    create_and_execute::<SET_6_xHL>, // 0xF6
    create_and_execute::<SET_6_A>,   // 0xF7
    create_and_execute::<SET_7_B>,   // 0xF8
    create_and_execute::<SET_7_C>,   // 0xF9
    create_and_execute::<SET_7_D>,   // 0xFA
    create_and_execute::<SET_7_E>,   // 0xFB
    create_and_execute::<SET_7_H>,   // 0xFC
    create_and_execute::<SET_7_L>,   // 0xFD
    create_and_execute::<SET_7_xHL>, // 0xFE
    create_and_execute::<SET_7_A>,   // 0xFF
];

fn create_and_execute<Op: OpConstruct + OpExecute>(
    opcode: Opcode, registers: &mut Registers, memory: &mut Memory) {
    let op = Op::new(opcode);
    op.execute(registers, memory);
}

fn execute_extended_opcode(
    opcode: Opcode, registers: &mut Registers, memory: &mut Memory) {
    let opcode_function = EXTENDED_OPCODE_MAP[opcode.b2 as usize];
    opcode_function(opcode, registers, memory);
}

#[derive(Copy, Clone)]
struct Opcode {
    b1: u8,
    b2: u8,
    b3: u8,
}

struct Registers {
    a: u8,
    f: u8,

    b: u8,
    c: u8,

    d: u8,
    e: u8,

    h: u8,
    l: u8,

    pc: u16,
    sp: u16,

    cycles_of_last_command: u8,
    interrupt_master_enable: bool,

    halt: bool,
}

macro_rules! generate_flag_getter_and_setter {
    ($name_getter:ident, $name_setter:ident, $val:expr) => {
        fn $name_getter(&self) -> bool {
            (self.f & $val) != 0
        }

        fn $name_setter(&mut self, value: bool) {
            if value {
                self.f |= $val;
            } else {
                self.f &= !$val;
            }
        }
    }
}

impl Registers {
    fn new() -> Registers {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0x0,
            sp: 0xFFFE,
            cycles_of_last_command: 0,
            interrupt_master_enable: false,
            halt: false,
        }
    }

    generate_flag_getter_and_setter!(get_zero, set_zero, 0x80);
    generate_flag_getter_and_setter!(get_operation, set_operation, 0x40);
    generate_flag_getter_and_setter!(get_halfcarry, set_halfcarry, 0x20);
    generate_flag_getter_and_setter!(get_carry, set_carry, 0x10);
}

trait OpConstruct {
    fn new(opcode: Opcode) -> Self;
}

trait OpExecute {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory);
}

fn to_u16(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) + l as u16
}

fn store_value_in_register_pair(value: u16, h: &mut u8, l: &mut u8) {
    *h = ((value & 0xFF00) >> 8) as u8;
    *l = (value & 0xFF) as u8;
}

fn decrement_register_pair(h: &mut u8, l: &mut u8) {
    let value = to_u16(*h, *l).wrapping_sub(1);
    store_value_in_register_pair(value, h, l);
}

fn increment_register_pair(h: &mut u8, l: &mut u8) {
    let value = to_u16(*h, *l).wrapping_add(1);
    store_value_in_register_pair(value, h, l);
}

fn test_carry_u16(a: u16, b: u16) -> (bool, bool) {
    let carry_mask = 0xFF;
    let halfcarry_mask = 0xF;
    let carry = (a & carry_mask) + (b & carry_mask) > carry_mask;
    let halfcarry = (a & halfcarry_mask) + (b & halfcarry_mask) > halfcarry_mask;
    (carry, halfcarry)
}

macro_rules! create_opcode_struct {
    ($name:ident) => {
        struct $name {
            _b2: u8,
            _b3: u8,
        }

        impl OpConstruct for $name {
            fn new(opcode: Opcode) -> Self {
                $name {
                    _b2: opcode.b2,
                    _b3: opcode.b3,
                }
            }
        }
    }
}

// Naming conventions for opcode classes:
// N: 8-bit immediate
// NN: 16-bit immediate
// (HL), (BC), etc. are named xHL, xBC, etc.

// 8-bit Loads /////////////////////////////////////////////////////////////////

// Load 8-bit immediate into register
macro_rules! ld_r_n {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                registers.$reg = self._b2;
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
ld_r_n!(
    a: LD_A_N,
    b: LD_B_N,
    c: LD_C_N,
    d: LD_D_N,
    e: LD_E_N,
    h: LD_H_N,
    l: LD_L_N
);

// Put value of r2 in r1
macro_rules! ld_r1_r2 {
    ($($reg1:ident, $reg2:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                registers.$reg1 = registers.$reg2;
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
ld_r1_r2!(
    a, a: LD_A_A,
    a, b: LD_A_B,
    a, c: LD_A_C,
    a, d: LD_A_D,
    a, e: LD_A_E,
    a, h: LD_A_H,
    a, l: LD_A_L,
    b, a: LD_B_A,
    b, b: LD_B_B,
    b, c: LD_B_C,
    b, d: LD_B_D,
    b, e: LD_B_E,
    b, h: LD_B_H,
    b, l: LD_B_L,
    c, a: LD_C_A,
    c, b: LD_C_B,
    c, c: LD_C_C,
    c, d: LD_C_D,
    c, e: LD_C_E,
    c, h: LD_C_H,
    c, l: LD_C_L,
    d, a: LD_D_A,
    d, b: LD_D_B,
    d, c: LD_D_C,
    d, d: LD_D_D,
    d, e: LD_D_E,
    d, h: LD_D_H,
    d, l: LD_D_L,
    e, a: LD_E_A,
    e, b: LD_E_B,
    e, c: LD_E_C,
    e, d: LD_E_D,
    e, e: LD_E_E,
    e, h: LD_E_H,
    e, l: LD_E_L,
    h, a: LD_H_A,
    h, b: LD_H_B,
    h, c: LD_H_C,
    h, d: LD_H_D,
    h, e: LD_H_E,
    h, h: LD_H_H,
    h, l: LD_H_L,
    l, a: LD_L_A,
    l, b: LD_L_B,
    l, c: LD_L_C,
    l, d: LD_L_D,
    l, e: LD_L_E,
    l, h: LD_L_H,
    l, l: LD_L_L
);

// Put value of (r2r3) in r1
macro_rules! ld_r_xrr {
    ($($reg1:ident, $reg2:ident, $reg3:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let address = to_u16(registers.$reg2, registers.$reg3);
                registers.$reg1 = memory.read_byte(address);
                registers.pc += 1;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
ld_r_xrr!(
    a, h, l: LD_A_xHL,
    b, h, l: LD_B_xHL,
    c, h, l: LD_C_xHL,
    d, h, l: LD_D_xHL,
    e, h, l: LD_E_xHL,
    h, h, l: LD_H_xHL,
    l, h, l: LD_L_xHL,
    a, b, c: LD_A_xBC,
    a, d, e: LD_A_xDE
);

// Put value of r3 in (r1r2)
macro_rules! ld_xhl_r {
    ($($reg1:ident, $reg2:ident, $reg3:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let address = to_u16(registers.$reg1, registers.$reg2);
                memory.write_byte(address, registers.$reg3);
                registers.pc += 1;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
ld_xhl_r!(
    h, l, a: LD_xHL_A,
    h, l, b: LD_xHL_B,
    h, l, c: LD_xHL_C,
    h, l, d: LD_xHL_D,
    h, l, e: LD_xHL_E,
    h, l, h: LD_xHL_H,
    h, l, l: LD_xHL_L,
    b, c, a: LD_xBC_A,
    d, e, a: LD_xDE_A
);

// Load 8-bit immediate into (HL)
create_opcode_struct!(LD_xHL_N);
impl OpExecute for LD_xHL_N {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        memory.write_byte(address, self._b2);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Load (nn) into A where nn is a 16-bit immediate
create_opcode_struct!(LD_A_xNN);
impl OpExecute for LD_A_xNN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(self._b3, self._b2);
        registers.a = memory.read_byte(address);
        registers.pc += 3;
        registers.cycles_of_last_command = 16;
    }
}

// Load A into (nn) where nn is a 16-bit immediate
create_opcode_struct!(LD_xNN_A);
impl OpExecute for LD_xNN_A {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(self._b3, self._b2);
        memory.write_byte(address, registers.a);
        registers.pc += 3;
        registers.cycles_of_last_command = 16;
    }
}

// Load (0xFF00 + C) into A
create_opcode_struct!(LDH_A_xC);
impl OpExecute for LDH_A_xC {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = 0xFF00 + registers.c as u16;
        registers.a = memory.read_byte(address);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load A into (0xFF00 + C)
create_opcode_struct!(LDH_xC_A);
impl OpExecute for LDH_xC_A {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = 0xFF00 + registers.c as u16;
        memory.write_byte(address, registers.a);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load (0xFF00 + N) into A
create_opcode_struct!(LDH_A_xN);
impl OpExecute for LDH_A_xN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = 0xFF00 + self._b2 as u16;
        registers.a = memory.read_byte(address);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Load A into (0xFF00 + N)
create_opcode_struct!(LDH_xN_A);
impl OpExecute for LDH_xN_A {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = 0xFF00 + self._b2 as u16;
        memory.write_byte(address, registers.a);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Load (HL) into A. Decrement HL.
create_opcode_struct!(LDD_A_xHL);
impl OpExecute for LDD_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        registers.a = memory.read_byte(address);
        decrement_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load A into (HL). Decrement HL.
create_opcode_struct!(LDD_xHL_A);
impl OpExecute for LDD_xHL_A {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        memory.write_byte(address, registers.a);
        decrement_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load (HL) into A. Increment HL.
create_opcode_struct!(LDI_A_xHL);
impl OpExecute for LDI_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        registers.a = memory.read_byte(address);
        increment_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load A into (HL). Increment HL.
create_opcode_struct!(LDI_xHL_A);
impl OpExecute for LDI_xHL_A {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        memory.write_byte(address, registers.a);
        increment_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// 16-bit Loads ////////////////////////////////////////////////////////////////

// Load 16-bit immediate into register pair
macro_rules! ld_rr_nn {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let value = to_u16(self._b3, self._b2);
                store_value_in_register_pair(value, &mut registers.$reg_high, &mut registers.$reg_low);
                registers.pc += 3;
                registers.cycles_of_last_command = 12;
            }
        }
    )*}
}
ld_rr_nn!(
    b, c: LD_BC_NN,
    d, e: LD_DE_NN,
    h, l: LD_HL_NN
);

// Load 16-bit immediate into stack pointer
create_opcode_struct!(LD_SP_NN);
impl OpExecute for LD_SP_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.sp = to_u16(self._b3, self._b2);
        registers.pc += 3;
        registers.cycles_of_last_command = 12;
    }
}

// Load 16-bit immediate into stack pointer
create_opcode_struct!(LD_SP_HL);
impl OpExecute for LD_SP_HL {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.sp = to_u16(registers.h, registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Put SP + n effective address into HL
create_opcode_struct!(LDHL_SP_N);
impl OpExecute for LDHL_SP_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let offset = self._b2 as i8 as u16;
        let address = registers.sp.wrapping_add(offset);
        store_value_in_register_pair(address, &mut registers.h, &mut registers.l);
        registers.set_zero(false);
        registers.set_operation(false);
        let (carry, halfcarry) = test_carry_u16(registers.sp, offset);
        registers.set_carry(carry);
        registers.set_halfcarry(halfcarry);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Save SP to given address
create_opcode_struct!(LD_xNN_SP);
impl OpExecute for LD_xNN_SP {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(self._b3, self._b2);
        memory.write_word(address, registers.sp);
        registers.pc += 3;
        registers.cycles_of_last_command = 20;
    }
}

// Push register pair onto stack. Decrement SP twice.
macro_rules! push_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                registers.sp -= 2;
                let value = to_u16(registers.$reg_high, registers.$reg_low);
                memory.write_word(registers.sp, value);
                registers.pc += 1;
                registers.cycles_of_last_command = 16;
            }
        }
    )*}
}
push_rr!(
    a, f: PUSH_AF,
    b, c: PUSH_BC,
    d, e: PUSH_DE,
    h, l: PUSH_HL
);

// Pop two bytes off stack into register pair. Increment SP twice.
macro_rules! pop_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let value = memory.read_word(registers.sp);
                store_value_in_register_pair(value, &mut registers.$reg_high, &mut registers.$reg_low);
                registers.sp += 2;
                registers.pc += 1;
                registers.cycles_of_last_command = 12;
            }
        }
    )*}
}
pop_rr!(
    b, c: POP_BC,
    d, e: POP_DE,
    h, l: POP_HL
);

// Pop two bytes off the stack with special handling for flags register. Increment SP twice.
create_opcode_struct!(POP_AF);
impl OpExecute for POP_AF {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let value = memory.read_word(registers.sp);
        registers.a = ((value & 0xFF00) >> 8) as u8;
        // Low nibble should always be zeroes!
        registers.f = (value & 0xF0) as u8;
        registers.sp += 2;
        registers.pc += 1;
        registers.cycles_of_last_command = 12;
    }
}

// Add register to A
macro_rules! add_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let a = registers.a;
                let r = registers.$reg;
                let sum = a.wrapping_add(r);
                registers.set_zero(sum == 0);
                registers.set_operation(false);
                registers.set_halfcarry((sum & 0xF) < (a & 0xF));
                registers.set_carry(sum < a);
                registers.a = sum;
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
add_a_r!(
    a: ADD_A_A,
    b: ADD_A_B,
    c: ADD_A_C,
    d: ADD_A_D,
    e: ADD_A_E,
    h: ADD_A_H,
    l: ADD_A_L
);

create_opcode_struct!(ADD_A_xHL);
impl OpExecute for ADD_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let sum = a.wrapping_add(val);
        registers.set_zero(sum == 0);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xF) < (a & 0xF));
        registers.set_carry(sum < a);
        registers.a = sum;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(ADD_A_N);
impl OpExecute for ADD_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let a = registers.a;
        let sum = a.wrapping_add(self._b2);
        registers.set_zero(sum == 0);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xF) < (a & 0xF));
        registers.set_carry(sum < a);
        registers.a = sum;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Add register + carry flag to A
macro_rules! adc_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let a = registers.a;
                let r = registers.$reg;
                let old_carry = registers.get_carry();
                let sum = a.wrapping_add(r).wrapping_add(old_carry as u8);
                let halfcarry = if old_carry {
                    (sum & 0xF) <= (a & 0xF)
                } else {
                    (sum & 0xF) < (a & 0xF)
                };
                let carry = if old_carry {
                    sum <= a
                } else {
                    sum < a
                };
                registers.set_zero(sum == 0);
                registers.set_operation(false);
                registers.set_halfcarry(halfcarry);
                registers.set_carry(carry);
                registers.a = sum;
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
adc_a_r!(
    a: ADC_A_A,
    b: ADC_A_B,
    c: ADC_A_C,
    d: ADC_A_D,
    e: ADC_A_E,
    h: ADC_A_H,
    l: ADC_A_L
);

create_opcode_struct!(ADC_A_xHL);
impl OpExecute for ADC_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let old_carry = registers.get_carry();
        let sum = a.wrapping_add(val).wrapping_add(old_carry as u8);
        let halfcarry = if old_carry {
            (sum & 0xF) <= (a & 0xF)
        } else {
            (sum & 0xF) < (a & 0xF)
        };
        let carry = if old_carry {
            sum <= a
        } else {
            sum < a
        };
        registers.set_zero(sum == 0);
        registers.set_operation(false);
        registers.set_halfcarry(halfcarry);
        registers.set_carry(carry);
        registers.a = sum;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(ADC_A_N);
impl OpExecute for ADC_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let a = registers.a;
        let old_carry = registers.get_carry();
        let sum = a.wrapping_add(self._b2).wrapping_add(old_carry as u8);
        let halfcarry = if old_carry {
            (sum & 0xF) <= (a & 0xF)
        } else {
            (sum & 0xF) < (a & 0xF)
        };
        let carry = if old_carry {
            sum <= a
        } else {
            sum < a
        };
        registers.set_zero(sum == 0);
        registers.set_operation(false);
        registers.set_halfcarry(halfcarry);
        registers.set_carry(carry);
        registers.a = sum;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Subtract register from A
macro_rules! sub_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let a = registers.a;
                let r = registers.$reg;
                let difference = a.wrapping_sub(r);
                registers.set_zero(difference == 0);
                registers.set_operation(true);
                registers.set_halfcarry((r & 0xF) > (a & 0xF));
                registers.set_carry(r > a);
                registers.a = difference;
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
sub_a_r!(
    a: SUB_A_A,
    b: SUB_A_B,
    c: SUB_A_C,
    d: SUB_A_D,
    e: SUB_A_E,
    h: SUB_A_H,
    l: SUB_A_L
);

create_opcode_struct!(SUB_A_xHL);
impl OpExecute for SUB_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let difference = a.wrapping_sub(val);
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) > (a & 0xF));
        registers.set_carry(val > a);
        registers.a = difference;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(SUB_A_N);
impl OpExecute for SUB_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let a = registers.a;
        let val = self._b2;
        let difference = a.wrapping_sub(val);
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) > (a & 0xF));
        registers.set_carry(val > a);
        registers.a = difference;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Subtract register + carry flag from A
macro_rules! sbc_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let a = registers.a;
                let r = registers.$reg;
                let carry = registers.get_carry() as u8;
                let difference = a.wrapping_sub(r).wrapping_sub(carry);
                let r_plus_c = r as u16 + carry as u16;
                registers.set_zero(difference == 0);
                registers.set_operation(true);
                registers.set_halfcarry((r & 0xF) + carry > (a & 0xF));
                registers.set_carry((r_plus_c) > (a as u16));
                registers.a = difference;
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
sbc_a_r!(
    a: SBC_A_A,
    b: SBC_A_B,
    c: SBC_A_C,
    d: SBC_A_D,
    e: SBC_A_E,
    h: SBC_A_H,
    l: SBC_A_L
);

create_opcode_struct!(SBC_A_xHL);
impl OpExecute for SBC_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = registers.get_carry() as u8;
        let difference = a.wrapping_sub(val).wrapping_sub(carry);
        let val_plus_c = val as u16 + carry as u16;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) + carry > (a & 0xF));
        registers.set_carry((val_plus_c) > (a as u16));
        registers.a = difference;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(SBC_A_N);
impl OpExecute for SBC_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let a = registers.a;
        let val = self._b2;
        let carry = registers.get_carry() as u8;
        let difference = a.wrapping_sub(val).wrapping_sub(carry);
        let val_plus_c = val as u16 + carry as u16;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) + carry > (a & 0xF));
        registers.set_carry((val_plus_c) > (a as u16));
        registers.a = difference;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Store logical AND of register and A in A
macro_rules! and_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                registers.a &= registers.$reg;
                let zero = registers.a == 0;
                registers.set_zero(zero);
                registers.set_operation(false);
                registers.set_halfcarry(true);
                registers.set_carry(false);
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
and_a_r!(
    a: AND_A_A,
    b: AND_A_B,
    c: AND_A_C,
    d: AND_A_D,
    e: AND_A_E,
    h: AND_A_H,
    l: AND_A_L
);

create_opcode_struct!(AND_A_xHL);
impl OpExecute for AND_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        registers.a &= val;
        let zero = registers.a == 0;
        registers.set_zero(zero);
        registers.set_operation(false);
        registers.set_halfcarry(true);
        registers.set_carry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(AND_A_N);
impl OpExecute for AND_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = self._b2;
        registers.a &= val;
        let zero = registers.a == 0;
        registers.set_zero(zero);
        registers.set_operation(false);
        registers.set_halfcarry(true);
        registers.set_carry(false);
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Store logical OR of register and A in A
macro_rules! or_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                registers.a |= registers.$reg;
                let zero = registers.a == 0;
                registers.set_zero(zero);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(false);
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
or_a_r!(
    a: OR_A_A,
    b: OR_A_B,
    c: OR_A_C,
    d: OR_A_D,
    e: OR_A_E,
    h: OR_A_H,
    l: OR_A_L
);

create_opcode_struct!(OR_A_xHL);
impl OpExecute for OR_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        registers.a |= val;
        let zero = registers.a == 0;
        registers.set_zero(zero);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(OR_A_N);
impl OpExecute for OR_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = self._b2;
        registers.a |= val;
        let zero = registers.a == 0;
        registers.set_zero(zero);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(false);
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Store logical XOR of register and A in A
macro_rules! xor_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                registers.a ^= registers.$reg;
                let zero = registers.a == 0;
                registers.set_zero(zero);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(false);
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
xor_a_r!(
    a: XOR_A_A,
    b: XOR_A_B,
    c: XOR_A_C,
    d: XOR_A_D,
    e: XOR_A_E,
    h: XOR_A_H,
    l: XOR_A_L
);

create_opcode_struct!(XOR_A_xHL);
impl OpExecute for XOR_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        registers.a ^= val;
        let zero = registers.a == 0;
        registers.set_zero(zero);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(XOR_A_N);
impl OpExecute for XOR_A_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = self._b2;
        registers.a ^= val;
        let zero = registers.a == 0;
        registers.set_zero(zero);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(false);
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Compare register with A
macro_rules! cp_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let a = registers.a;
                let r = registers.$reg;
                let difference = a.wrapping_sub(r);
                registers.set_zero(difference == 0);
                registers.set_operation(true);
                registers.set_halfcarry((r & 0xF) > (a & 0xF));
                registers.set_carry(r > a);
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
cp_r!(
    a: CP_A,
    b: CP_B,
    c: CP_C,
    d: CP_D,
    e: CP_E,
    h: CP_H,
    l: CP_L
);

create_opcode_struct!(CP_xHL);
impl OpExecute for CP_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let difference = a.wrapping_sub(val);
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) > (a & 0xF));
        registers.set_carry(val > a);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(CP_N);
impl OpExecute for CP_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let a = registers.a;
        let val = self._b2;
        let difference = a.wrapping_sub(val);
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) > (a & 0xF));
        registers.set_carry(val > a);
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Increment register
macro_rules! inc_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let halfcarry = (val & 0xF) == 0xF;
                registers.set_halfcarry(halfcarry);
                let new_val = val.wrapping_add(1);
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
inc_r!(
    a: INC_A,
    b: INC_B,
    c: INC_C,
    d: INC_D,
    e: INC_E,
    h: INC_H,
    l: INC_L
);

create_opcode_struct!(INC_xHL);
impl OpExecute for INC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let halfcarry = (val & 0xF) == 0xF;
        registers.set_halfcarry(halfcarry);
        let new_val = val.wrapping_add(1);
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 12;
    }
}

// Decrement register
macro_rules! dec_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let new_val = val.wrapping_sub(1);
                registers.$reg = new_val;
                let borrow = (new_val & 0xF) == 0xF;
                registers.set_halfcarry(borrow); // TODO: richtig??
                registers.set_zero(new_val == 0);
                registers.set_operation(true);
                registers.pc += 1;
                registers.cycles_of_last_command = 4;
            }
        }
    )*}
}
dec_r!(
    a: DEC_A,
    b: DEC_B,
    c: DEC_C,
    d: DEC_D,
    e: DEC_E,
    h: DEC_H,
    l: DEC_L
);

create_opcode_struct!(DEC_xHL);
impl OpExecute for DEC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let new_val = val.wrapping_sub(1);
        memory.write_byte(address, new_val);
        let borrow = (new_val & 0xF) == 0xF;
        registers.set_halfcarry(borrow); // TODO: richtig??
        registers.set_zero(new_val == 0);
        registers.set_operation(true);
        registers.pc += 1;
        registers.cycles_of_last_command = 12;
    }
}

// Add register pair to HL
macro_rules! add_hl_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let hl = to_u16(registers.h, registers.l);
                let rr = to_u16(registers.$reg_high, registers.$reg_low);
                let sum = hl.wrapping_add(rr);
                store_value_in_register_pair(sum, &mut registers.h, &mut registers.l);
                registers.set_operation(false);
                registers.set_halfcarry((sum & 0xFFF) < (hl & 0xFFF));
                registers.set_carry(sum < hl);
                registers.pc += 1;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
add_hl_rr!(
    b, c: ADD_HL_BC,
    d, e: ADD_HL_DE,
    h, l: ADD_HL_HL
);

create_opcode_struct!(ADD_HL_SP);
impl OpExecute for ADD_HL_SP {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let hl = to_u16(registers.h, registers.l);
        let sum = hl.wrapping_add(registers.sp);
        store_value_in_register_pair(sum, &mut registers.h, &mut registers.l);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xFFF) < (hl & 0xFFF));
        registers.set_carry(sum < hl);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(ADD_SP_N);
impl OpExecute for ADD_SP_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let sp = registers.sp;
        let val = self._b2 as i8 as u16;
        let sum = sp.wrapping_add(val);
        registers.sp = sum;
        registers.set_zero(false);
        registers.set_operation(false);
        let (carry, halfcarry) = test_carry_u16(sp, val);
        registers.set_carry(carry);
        registers.set_halfcarry(halfcarry);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Increment register pair
macro_rules! inc_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = to_u16(registers.$reg_high, registers.$reg_low);
                let new_val = val.wrapping_add(1);
                store_value_in_register_pair(new_val, &mut registers.$reg_high, &mut registers.$reg_low);
                registers.pc += 1;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
inc_rr!(
    b, c: INC_BC,
    d, e: INC_DE,
    h, l: INC_HL
);

create_opcode_struct!(INC_SP);
impl OpExecute for INC_SP {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.sp = registers.sp.wrapping_add(1);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Decrement register pair
macro_rules! dec_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = to_u16(registers.$reg_high, registers.$reg_low);
                let new_val = val.wrapping_sub(1);
                store_value_in_register_pair(new_val, &mut registers.$reg_high, &mut registers.$reg_low);
                registers.pc += 1;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
dec_rr!(
    b, c: DEC_BC,
    d, e: DEC_DE,
    h, l: DEC_HL
);

create_opcode_struct!(DEC_SP);
impl OpExecute for DEC_SP {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.sp = registers.sp.wrapping_sub(1);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Swap upper and lower nibbles of register
macro_rules! swap_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let high = (val & 0xF0) >> 4;
                let low = val & 0xF;
                let new_val = (low << 4) + high;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(false);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
swap_r!(
    a: SWAP_A,
    b: SWAP_B,
    c: SWAP_C,
    d: SWAP_D,
    e: SWAP_E,
    h: SWAP_H,
    l: SWAP_L
);

create_opcode_struct!(SWAP_xHL);
impl OpExecute for SWAP_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let high = (val & 0xF0) >> 4;
        let low = val & 0xF;
        let new_val = (low << 4) + high;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(false);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// TODO: Diesen Opcode nochmal prüfen
// BCD correction for register A
create_opcode_struct!(DAA);
impl OpExecute for DAA {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let old_halfcarry = registers.get_halfcarry();
        let old_carry = registers.get_carry();
        let operation = registers.get_operation();
        let mut val = registers.a;
        let low_nibble = val & 0xF;
        let mut carry = false;
        if !operation {
            if old_carry || val > 0x99 {
                val = val.wrapping_add(0x60);
                carry = true;
            }
            if old_halfcarry || low_nibble > 0x9 {
                val = val.wrapping_add(0x6);
            }
        } else if old_carry {
            carry = true;
            val = val.wrapping_add(
                if old_halfcarry { 0x9A }
                else { 0xA0 }
            );
        } else if old_halfcarry {
            val = val.wrapping_add(0xFA);
        }
        registers.a = val;
        registers.set_zero(val == 0);
        registers.set_halfcarry(false);
        registers.set_carry(carry);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Complement A register
create_opcode_struct!(CPL);
impl OpExecute for CPL {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.a ^= 0xFF;
        registers.set_operation(true);
        registers.set_halfcarry(true);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Complement carry flag
create_opcode_struct!(CCF);
impl OpExecute for CCF {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let carry = registers.get_carry();
        registers.set_carry(!carry);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Set carry flag
create_opcode_struct!(SCF);
impl OpExecute for SCF {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.set_carry(true);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// No operation
create_opcode_struct!(NOP);
impl OpExecute for NOP {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Power down CPU until an interrupt occurs
create_opcode_struct!(HALT);
impl OpExecute for HALT {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.halt = true;
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Halt CPU & LCD display until button pressed
create_opcode_struct!(STOP);
impl OpExecute for STOP {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        //TODO: Implement
        println!("STOP is not yet implemented!");
        registers.pc += 2;
        registers.cycles_of_last_command = 4;
    }
}

// Disables interrupts after the next instruction is executed
create_opcode_struct!(DI);
impl OpExecute for DI {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.interrupt_master_enable = false;
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Enables interrupts after the next instruction is executed
create_opcode_struct!(EI);
impl OpExecute for EI {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.interrupt_master_enable = true;
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A left
create_opcode_struct!(RLCA);
impl OpExecute for RLCA {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = registers.a;
        let carry = (val & 0x80) >> 7;
        let new_val = (val << 1) + carry;
        registers.a = new_val;
        registers.set_zero(false);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry != 0);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A left through carry flag
create_opcode_struct!(RLA);
impl OpExecute for RLA {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = registers.a;
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x80) != 0;
        let new_val = (val << 1) + carry_in;
        registers.a = new_val;
        registers.set_zero(false);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry_out);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A right
create_opcode_struct!(RRCA);
impl OpExecute for RRCA {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = registers.a;
        let carry = val & 0x1;
        let new_val = (val >> 1) + carry * 0x80;
        registers.a = new_val;
        registers.set_zero(false);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry != 0);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A right through carry flag
create_opcode_struct!(RRA);
impl OpExecute for RRA {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let val = registers.a;
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x1) != 0;
        let new_val = (val >> 1) + carry_in * 0x80;
        registers.a = new_val;
        registers.set_zero(false);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry_out);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate register left
macro_rules! rlc_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry = (val & 0x80) >> 7;
                let new_val = (val << 1) + carry;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry != 0);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
rlc_r!(
    a: RLC_A,
    b: RLC_B,
    c: RLC_C,
    d: RLC_D,
    e: RLC_E,
    h: RLC_H,
    l: RLC_L
);

create_opcode_struct!(RLC_xHL);
impl OpExecute for RLC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = (val & 0x80) >> 7;
        let new_val = (val << 1) + carry;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry != 0);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Rotate register left through carry flag
macro_rules! rl_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry_in = registers.get_carry() as u8;
                let carry_out = (val & 0x80) != 0;
                let new_val = (val << 1) + carry_in;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry_out);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
rl_r!(
    a: RL_A,
    b: RL_B,
    c: RL_C,
    d: RL_D,
    e: RL_E,
    h: RL_H,
    l: RL_L
);

create_opcode_struct!(RL_xHL);
impl OpExecute for RL_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x80) != 0;
        let new_val = (val << 1) + carry_in;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry_out);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Rotate register right
macro_rules! rrc_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry = val & 0x1;
                let new_val = (val >> 1) + carry * 0x80;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry != 0);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
rrc_r!(
    a: RRC_A,
    b: RRC_B,
    c: RRC_C,
    d: RRC_D,
    e: RRC_E,
    h: RRC_H,
    l: RRC_L
);

create_opcode_struct!(RRC_xHL);
impl OpExecute for RRC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = val & 0x1;
        let new_val = (val >> 1) + carry * 0x80;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry != 0);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Rotate register right through carry flag
macro_rules! rr_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry_in = registers.get_carry() as u8;
                let carry_out = (val & 0x1) != 0;
                let new_val = (val >> 1) + carry_in * 0x80;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry_out);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
rr_r!(
    a: RR_A,
    b: RR_B,
    c: RR_C,
    d: RR_D,
    e: RR_E,
    h: RR_H,
    l: RR_L
);

create_opcode_struct!(RR_xHL);
impl OpExecute for RR_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x1) != 0;
        let new_val = (val >> 1) + carry_in * 0x80;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry_out);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Shift register left into carry flag. LSB set to 0.
macro_rules! sla_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry = (val & 0x80) != 0;
                let new_val = val << 1;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
sla_r!(
    a: SLA_A,
    b: SLA_B,
    c: SLA_C,
    d: SLA_D,
    e: SLA_E,
    h: SLA_H,
    l: SLA_L
);

create_opcode_struct!(SLA_xHL);
impl OpExecute for SLA_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = (val & 0x80) != 0;
        let new_val = val << 1;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Shift register right into carry flag. MSB doesn’t change.
macro_rules! sra_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry = (val & 0x1) != 0;
                let msb = val & 0x80;
                let mut new_val = val >> 1;
                new_val |= msb;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
sra_r!(
    a: SRA_A,
    b: SRA_B,
    c: SRA_C,
    d: SRA_D,
    e: SRA_E,
    h: SRA_H,
    l: SRA_L
);

create_opcode_struct!(SRA_xHL);
impl OpExecute for SRA_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = (val & 0x1) != 0;
        let msb = val & 0x80;
        let mut new_val = val >> 1;
        new_val |= msb;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Shift register right into carry flag. MSB set to 0.
macro_rules! srl_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let carry = (val & 0x1) != 0;
                let new_val = val >> 1;
                registers.$reg = new_val;
                registers.set_zero(new_val == 0);
                registers.set_operation(false);
                registers.set_halfcarry(false);
                registers.set_carry(carry);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
srl_r!(
    a: SRL_A,
    b: SRL_B,
    c: SRL_C,
    d: SRL_D,
    e: SRL_E,
    h: SRL_H,
    l: SRL_L
);

create_opcode_struct!(SRL_xHL);
impl OpExecute for SRL_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = (val & 0x1) != 0;
        let new_val = val >> 1;
        memory.write_byte(address, new_val);
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry);
        registers.pc += 2;
        registers.cycles_of_last_command = 16;
    }
}

// Test bit b in register r
macro_rules! bit_b_r {
    ($($bit:expr, $reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let reg = registers.$reg;
                let bit = (0x1 << $bit) & reg != 0;
                registers.set_zero(!bit);
                registers.set_operation(false);
                registers.set_halfcarry(true);
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
bit_b_r!(
    0, a: BIT_0_A,
    0, b: BIT_0_B,
    0, c: BIT_0_C,
    0, d: BIT_0_D,
    0, e: BIT_0_E,
    0, h: BIT_0_H,
    0, l: BIT_0_L,
    1, a: BIT_1_A,
    1, b: BIT_1_B,
    1, c: BIT_1_C,
    1, d: BIT_1_D,
    1, e: BIT_1_E,
    1, h: BIT_1_H,
    1, l: BIT_1_L,
    2, a: BIT_2_A,
    2, b: BIT_2_B,
    2, c: BIT_2_C,
    2, d: BIT_2_D,
    2, e: BIT_2_E,
    2, h: BIT_2_H,
    2, l: BIT_2_L,
    3, a: BIT_3_A,
    3, b: BIT_3_B,
    3, c: BIT_3_C,
    3, d: BIT_3_D,
    3, e: BIT_3_E,
    3, h: BIT_3_H,
    3, l: BIT_3_L,
    4, a: BIT_4_A,
    4, b: BIT_4_B,
    4, c: BIT_4_C,
    4, d: BIT_4_D,
    4, e: BIT_4_E,
    4, h: BIT_4_H,
    4, l: BIT_4_L,
    5, a: BIT_5_A,
    5, b: BIT_5_B,
    5, c: BIT_5_C,
    5, d: BIT_5_D,
    5, e: BIT_5_E,
    5, h: BIT_5_H,
    5, l: BIT_5_L,
    6, a: BIT_6_A,
    6, b: BIT_6_B,
    6, c: BIT_6_C,
    6, d: BIT_6_D,
    6, e: BIT_6_E,
    6, h: BIT_6_H,
    6, l: BIT_6_L,
    7, a: BIT_7_A,
    7, b: BIT_7_B,
    7, c: BIT_7_C,
    7, d: BIT_7_D,
    7, e: BIT_7_E,
    7, h: BIT_7_H,
    7, l: BIT_7_L
);

macro_rules! bit_b_hl {
    ($($bit:expr, $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let address = to_u16(registers.h, registers.l);
                let val = memory.read_byte(address);
                let bit = (0x1 << $bit) & val != 0;
                registers.set_zero(!bit);
                registers.set_operation(false);
                registers.set_halfcarry(true);
                registers.pc += 2;
                registers.cycles_of_last_command = 16;
            }
        }
    )*}
}
bit_b_hl!(
    0, BIT_0_xHL,
    1, BIT_1_xHL,
    2, BIT_2_xHL,
    3, BIT_3_xHL,
    4, BIT_4_xHL,
    5, BIT_5_xHL,
    6, BIT_6_xHL,
    7, BIT_7_xHL
);

// Set bit b in register r
macro_rules! set_b_r {
    ($($bit:expr, $reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let bit = 0x1 << $bit;
                let new_val = val | bit;
                registers.$reg = new_val;
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
set_b_r!(
    0, a: SET_0_A,
    0, b: SET_0_B,
    0, c: SET_0_C,
    0, d: SET_0_D,
    0, e: SET_0_E,
    0, h: SET_0_H,
    0, l: SET_0_L,
    1, a: SET_1_A,
    1, b: SET_1_B,
    1, c: SET_1_C,
    1, d: SET_1_D,
    1, e: SET_1_E,
    1, h: SET_1_H,
    1, l: SET_1_L,
    2, a: SET_2_A,
    2, b: SET_2_B,
    2, c: SET_2_C,
    2, d: SET_2_D,
    2, e: SET_2_E,
    2, h: SET_2_H,
    2, l: SET_2_L,
    3, a: SET_3_A,
    3, b: SET_3_B,
    3, c: SET_3_C,
    3, d: SET_3_D,
    3, e: SET_3_E,
    3, h: SET_3_H,
    3, l: SET_3_L,
    4, a: SET_4_A,
    4, b: SET_4_B,
    4, c: SET_4_C,
    4, d: SET_4_D,
    4, e: SET_4_E,
    4, h: SET_4_H,
    4, l: SET_4_L,
    5, a: SET_5_A,
    5, b: SET_5_B,
    5, c: SET_5_C,
    5, d: SET_5_D,
    5, e: SET_5_E,
    5, h: SET_5_H,
    5, l: SET_5_L,
    6, a: SET_6_A,
    6, b: SET_6_B,
    6, c: SET_6_C,
    6, d: SET_6_D,
    6, e: SET_6_E,
    6, h: SET_6_H,
    6, l: SET_6_L,
    7, a: SET_7_A,
    7, b: SET_7_B,
    7, c: SET_7_C,
    7, d: SET_7_D,
    7, e: SET_7_E,
    7, h: SET_7_H,
    7, l: SET_7_L
);

macro_rules! set_b_hl {
    ($($bit:expr, $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let address = to_u16(registers.h, registers.l);
                let val = memory.read_byte(address);
                let bit = 0x1 << $bit;
                let new_val = val | bit;
                memory.write_byte(address, new_val);
                registers.pc += 2;
                registers.cycles_of_last_command = 16;
            }
        }
    )*}
}
set_b_hl!(
    0, SET_0_xHL,
    1, SET_1_xHL,
    2, SET_2_xHL,
    3, SET_3_xHL,
    4, SET_4_xHL,
    5, SET_5_xHL,
    6, SET_6_xHL,
    7, SET_7_xHL
);

// Reset bit b in register r
macro_rules! res_b_r {
    ($($bit:expr, $reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
                let val = registers.$reg;
                let bit = 0x1 << $bit;
                let new_val = val & !bit;
                registers.$reg = new_val;
                registers.pc += 2;
                registers.cycles_of_last_command = 8;
            }
        }
    )*}
}
res_b_r!(
    0, a: RES_0_A,
    0, b: RES_0_B,
    0, c: RES_0_C,
    0, d: RES_0_D,
    0, e: RES_0_E,
    0, h: RES_0_H,
    0, l: RES_0_L,
    1, a: RES_1_A,
    1, b: RES_1_B,
    1, c: RES_1_C,
    1, d: RES_1_D,
    1, e: RES_1_E,
    1, h: RES_1_H,
    1, l: RES_1_L,
    2, a: RES_2_A,
    2, b: RES_2_B,
    2, c: RES_2_C,
    2, d: RES_2_D,
    2, e: RES_2_E,
    2, h: RES_2_H,
    2, l: RES_2_L,
    3, a: RES_3_A,
    3, b: RES_3_B,
    3, c: RES_3_C,
    3, d: RES_3_D,
    3, e: RES_3_E,
    3, h: RES_3_H,
    3, l: RES_3_L,
    4, a: RES_4_A,
    4, b: RES_4_B,
    4, c: RES_4_C,
    4, d: RES_4_D,
    4, e: RES_4_E,
    4, h: RES_4_H,
    4, l: RES_4_L,
    5, a: RES_5_A,
    5, b: RES_5_B,
    5, c: RES_5_C,
    5, d: RES_5_D,
    5, e: RES_5_E,
    5, h: RES_5_H,
    5, l: RES_5_L,
    6, a: RES_6_A,
    6, b: RES_6_B,
    6, c: RES_6_C,
    6, d: RES_6_D,
    6, e: RES_6_E,
    6, h: RES_6_H,
    6, l: RES_6_L,
    7, a: RES_7_A,
    7, b: RES_7_B,
    7, c: RES_7_C,
    7, d: RES_7_D,
    7, e: RES_7_E,
    7, h: RES_7_H,
    7, l: RES_7_L
);

macro_rules! res_b_hl {
    ($($bit:expr, $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let address = to_u16(registers.h, registers.l);
                let val = memory.read_byte(address);
                let bit = 0x1 << $bit;
                let new_val = val & !bit;
                memory.write_byte(address, new_val);
                registers.pc += 2;
                registers.cycles_of_last_command = 16;
            }
        }
    )*}
}
res_b_hl!(
    0, RES_0_xHL,
    1, RES_1_xHL,
    2, RES_2_xHL,
    3, RES_3_xHL,
    4, RES_4_xHL,
    5, RES_5_xHL,
    6, RES_6_xHL,
    7, RES_7_xHL
);

// Jump to address nn
create_opcode_struct!(JP_NN);
impl OpExecute for JP_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let address = to_u16(self._b3, self._b2);
        registers.pc = address;
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if Z flag is reset
create_opcode_struct!(JP_NZ_NN);
impl OpExecute for JP_NZ_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        if !registers.get_zero() {
            let address = to_u16(self._b3, self._b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if Z flag is set
create_opcode_struct!(JP_Z_NN);
impl OpExecute for JP_Z_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        if registers.get_zero() {
            let address = to_u16(self._b3, self._b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if C flag is reset
create_opcode_struct!(JP_NC_NN);
impl OpExecute for JP_NC_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        if !registers.get_carry() {
            let address = to_u16(self._b3, self._b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if C flag is set
create_opcode_struct!(JP_C_NN);
impl OpExecute for JP_C_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        if registers.get_carry() {
            let address = to_u16(self._b3, self._b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address in HL
create_opcode_struct!(JP_xHL);
impl OpExecute for JP_xHL {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        let address = to_u16(registers.h, registers.l);
        registers.pc = address;
        registers.cycles_of_last_command = 4;
    }
}

// Jump to n + current address
create_opcode_struct!(JR_N);
impl OpExecute for JR_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.pc += 2;
        registers.pc = ((registers.pc as i32) + self._b2 as i8 as i32) as u16;
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if Z flag is reset
create_opcode_struct!(JR_NZ_N);
impl OpExecute for JR_NZ_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.pc += 2;
        if !registers.get_zero() {
            registers.pc = ((registers.pc as i32) + self._b2 as i8 as i32) as u16;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if Z flag is set
create_opcode_struct!(JR_Z_N);
impl OpExecute for JR_Z_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.pc += 2;
        if registers.get_zero() {
            registers.pc = ((registers.pc as i32) + self._b2 as i8 as i32) as u16;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if C flag is reset
create_opcode_struct!(JR_NC_N);
impl OpExecute for JR_NC_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.pc += 2;
        if !registers.get_carry() {
            registers.pc = ((registers.pc as i32) + self._b2 as i8 as i32) as u16;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if C flag is set
create_opcode_struct!(JR_C_N);
impl OpExecute for JR_C_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
        registers.pc += 2;
        if registers.get_carry() {
            registers.pc = ((registers.pc as i32) + self._b2 as i8 as i32) as u16;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Push address of next instruction onto stack and then jump to address nn
create_opcode_struct!(CALL_NN);
impl OpExecute for CALL_NN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = to_u16(self._b3, self._b2);
        registers.sp -= 2;
        memory.write_word(registers.sp, registers.pc + 3);
        registers.pc = address;
        registers.cycles_of_last_command = 12;
    }
}

// Call address nn if Z flag is reset
create_opcode_struct!(CALL_NZ_NN);
impl OpExecute for CALL_NZ_NN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if !registers.get_zero() {
            let address = to_u16(self._b3, self._b2);
            registers.sp -= 2;
            memory.write_word(registers.sp, registers.pc + 3);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Call address nn if Z flag is set
create_opcode_struct!(CALL_Z_NN);
impl OpExecute for CALL_Z_NN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if registers.get_zero() {
            let address = to_u16(self._b3, self._b2);
            registers.sp -= 2;
            memory.write_word(registers.sp, registers.pc + 3);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Call address nn if C flag is reset
create_opcode_struct!(CALL_NC_NN);
impl OpExecute for CALL_NC_NN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if !registers.get_carry() {
            let address = to_u16(self._b3, self._b2);
            registers.sp -= 2;
            memory.write_word(registers.sp, registers.pc + 3);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Call address nn if C flag is set
create_opcode_struct!(CALL_C_NN);
impl OpExecute for CALL_C_NN {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if registers.get_carry() {
            let address = to_u16(self._b3, self._b2);
            registers.sp -= 2;
            memory.write_word(registers.sp, registers.pc + 3);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

macro_rules! rst_n {
    ($($address:expr, $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl OpExecute for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                registers.sp -= 2;
                memory.write_word(registers.sp, registers.pc + 1);
                registers.pc = $address;
                registers.cycles_of_last_command = 32;
            }
        }
    )*}
}
rst_n!(
    0x00, RST_0x00,
    0x08, RST_0x08,
    0x10, RST_0x10,
    0x18, RST_0x18,
    0x20, RST_0x20,
    0x28, RST_0x28,
    0x30, RST_0x30,
    0x38, RST_0x38
);

// Return
create_opcode_struct!(RET);
impl OpExecute for RET {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let address = memory.read_word(registers.sp);
        registers.sp += 2;
        registers.pc = address;
        registers.cycles_of_last_command = 8;
    }
}

// Return if Z flag is reset
create_opcode_struct!(RET_NZ);
impl OpExecute for RET_NZ {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if !registers.get_zero() {
            let address = memory.read_word(registers.sp);
            registers.sp += 2;
            registers.pc = address;
        } else {
            registers.pc += 1;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Return if Z flag is set
create_opcode_struct!(RET_Z);
impl OpExecute for RET_Z {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if registers.get_zero() {
            let address = memory.read_word(registers.sp);
            registers.sp += 2;
            registers.pc = address;
        } else {
            registers.pc += 1;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Return if C flag is reset
create_opcode_struct!(RET_NC);
impl OpExecute for RET_NC {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if !registers.get_carry() {
            let address = memory.read_word(registers.sp);
            registers.sp += 2;
            registers.pc = address;
        } else {
            registers.pc += 1;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Return if C flag is set
create_opcode_struct!(RET_C);
impl OpExecute for RET_C {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        if registers.get_carry() {
            let address = memory.read_word(registers.sp);
            registers.sp += 2;
            registers.pc = address;
        } else {
            registers.pc += 1;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Return, then enable interrupts
create_opcode_struct!(RETI);
impl OpExecute for RETI {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        registers.interrupt_master_enable = true;
        let address = memory.read_word(registers.sp);
        registers.sp += 2;
        registers.pc = address;
        registers.cycles_of_last_command = 8;
    }
}

// Unused operation
create_opcode_struct!(XX);
impl OpExecute for XX {
    fn execute(&self, _registers: &mut Registers, _memory: &mut Memory) {
        panic!("Tried to call unused opcode");
    }
}
