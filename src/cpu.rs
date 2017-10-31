use memory::Memory;
use std::num::Wrapping;

pub struct Cpu<M: Memory> {
    registers: Registers,
    memory: M,
}

impl<M: Memory> Cpu<M> {
    pub fn new(memory: M) -> Cpu<M> {
        Cpu {
            registers: Registers::new(),
            memory,
        }
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);
    }

    //pub fn decrement_timers(&mut self) {
    //    if self.registers.delay_timer > 0 {
    //        self.registers.delay_timer -= 1;
    //    }
    //    if self.registers.sound_timer > 0 {
    //        self.registers.sound_timer -= 1;
    //        if self.registers.sound_timer == 0 {
    //            self.audio_device.pause();
    //        }
    //    }
    //}

    fn fetch_opcode(&self) -> Opcode {
        let pc = self.registers.pc;
        Opcode {
            b1: self.memory.read_byte(pc),
            b2: self.memory.read_byte(pc + 1),
            b3: self.memory.read_byte(pc + 2),
        }
    }

    fn execute_opcode(&mut self, opcode: Opcode) {
        match opcode.b1 {
            0x3E => self.create_and_execute::<LD_A_N>(opcode),
            0x06 => self.create_and_execute::<LD_B_N>(opcode),
            0x0E => self.create_and_execute::<LD_C_N>(opcode),
            0x16 => self.create_and_execute::<LD_D_N>(opcode),
            0x1E => self.create_and_execute::<LD_E_N>(opcode),
            0x26 => self.create_and_execute::<LD_H_N>(opcode),
            0x2E => self.create_and_execute::<LD_L_N>(opcode),

            0x7F => self.create_and_execute::<LD_A_A>(opcode),
            0x78 => self.create_and_execute::<LD_A_B>(opcode),
            0x79 => self.create_and_execute::<LD_A_C>(opcode),
            0x7A => self.create_and_execute::<LD_A_D>(opcode),
            0x7B => self.create_and_execute::<LD_A_E>(opcode),
            0x7C => self.create_and_execute::<LD_A_H>(opcode),
            0x7D => self.create_and_execute::<LD_A_L>(opcode),

            0x47 => self.create_and_execute::<LD_B_A>(opcode),
            0x40 => self.create_and_execute::<LD_B_B>(opcode),
            0x41 => self.create_and_execute::<LD_B_C>(opcode),
            0x42 => self.create_and_execute::<LD_B_D>(opcode),
            0x43 => self.create_and_execute::<LD_B_E>(opcode),
            0x44 => self.create_and_execute::<LD_B_H>(opcode),
            0x45 => self.create_and_execute::<LD_B_L>(opcode),

            0x4F => self.create_and_execute::<LD_C_A>(opcode),
            0x48 => self.create_and_execute::<LD_C_B>(opcode),
            0x49 => self.create_and_execute::<LD_C_C>(opcode),
            0x4A => self.create_and_execute::<LD_C_D>(opcode),
            0x4B => self.create_and_execute::<LD_C_E>(opcode),
            0x4C => self.create_and_execute::<LD_C_H>(opcode),
            0x4D => self.create_and_execute::<LD_C_L>(opcode),

            0x57 => self.create_and_execute::<LD_D_A>(opcode),
            0x50 => self.create_and_execute::<LD_D_B>(opcode),
            0x51 => self.create_and_execute::<LD_D_C>(opcode),
            0x52 => self.create_and_execute::<LD_D_D>(opcode),
            0x53 => self.create_and_execute::<LD_D_E>(opcode),
            0x54 => self.create_and_execute::<LD_D_H>(opcode),
            0x55 => self.create_and_execute::<LD_D_L>(opcode),

            0x5F => self.create_and_execute::<LD_E_A>(opcode),
            0x58 => self.create_and_execute::<LD_E_B>(opcode),
            0x59 => self.create_and_execute::<LD_E_C>(opcode),
            0x5A => self.create_and_execute::<LD_E_D>(opcode),
            0x5B => self.create_and_execute::<LD_E_E>(opcode),
            0x5C => self.create_and_execute::<LD_E_H>(opcode),
            0x5D => self.create_and_execute::<LD_E_L>(opcode),

            0x67 => self.create_and_execute::<LD_H_A>(opcode),
            0x60 => self.create_and_execute::<LD_H_B>(opcode),
            0x61 => self.create_and_execute::<LD_H_C>(opcode),
            0x62 => self.create_and_execute::<LD_H_D>(opcode),
            0x63 => self.create_and_execute::<LD_H_E>(opcode),
            0x64 => self.create_and_execute::<LD_H_H>(opcode),
            0x65 => self.create_and_execute::<LD_H_L>(opcode),

            0x6F => self.create_and_execute::<LD_L_A>(opcode),
            0x68 => self.create_and_execute::<LD_L_B>(opcode),
            0x69 => self.create_and_execute::<LD_L_C>(opcode),
            0x6A => self.create_and_execute::<LD_L_D>(opcode),
            0x6B => self.create_and_execute::<LD_L_E>(opcode),
            0x6C => self.create_and_execute::<LD_L_H>(opcode),
            0x6D => self.create_and_execute::<LD_L_L>(opcode),

            0x7E => self.create_and_execute::<LD_A_xHL>(opcode),
            0x46 => self.create_and_execute::<LD_B_xHL>(opcode),
            0x4E => self.create_and_execute::<LD_C_xHL>(opcode),
            0x56 => self.create_and_execute::<LD_D_xHL>(opcode),
            0x5E => self.create_and_execute::<LD_E_xHL>(opcode),
            0x66 => self.create_and_execute::<LD_H_xHL>(opcode),
            0x6E => self.create_and_execute::<LD_L_xHL>(opcode),
            0x0A => self.create_and_execute::<LD_A_xBC>(opcode),
            0x1A => self.create_and_execute::<LD_A_xDE>(opcode),

            0x70 => self.create_and_execute::<LD_xHL_B>(opcode),
            0x71 => self.create_and_execute::<LD_xHL_C>(opcode),
            0x72 => self.create_and_execute::<LD_xHL_D>(opcode),
            0x73 => self.create_and_execute::<LD_xHL_E>(opcode),
            0x74 => self.create_and_execute::<LD_xHL_H>(opcode),
            0x75 => self.create_and_execute::<LD_xHL_L>(opcode),
            0x02 => self.create_and_execute::<LD_xBC_A>(opcode),
            0x12 => self.create_and_execute::<LD_xDE_A>(opcode),

            0x36 => self.create_and_execute::<LD_xHL_N>(opcode),

            0xFA => self.create_and_execute::<LD_A_xNN>(opcode),

            0xF2 => self.create_and_execute::<LDH_A_xC>(opcode),
            0xE2 => self.create_and_execute::<LDH_xC_A>(opcode),

            0x3A => self.create_and_execute::<LDD_A_xHL>(opcode),
            0x32 => self.create_and_execute::<LDD_xHL_A>(opcode),

            0x2A => self.create_and_execute::<LDI_A_xHL>(opcode),
            0x22 => self.create_and_execute::<LDI_xHL_A>(opcode),

            0xF0 => self.create_and_execute::<LDH_A_xN>(opcode),
            0xE0 => self.create_and_execute::<LDH_xN_A>(opcode),

            0x01 => self.create_and_execute::<LD_BC_NN>(opcode),
            0x11 => self.create_and_execute::<LD_DE_NN>(opcode),
            0x21 => self.create_and_execute::<LD_HL_NN>(opcode),

            0x31 => self.create_and_execute::<LD_SP_NN>(opcode),
            0xF9 => self.create_and_execute::<LD_SP_HL>(opcode),
            0xF8 => self.create_and_execute::<LDHL_SP_N>(opcode),
            0x08 => self.create_and_execute::<LD_xNN_SP>(opcode),

            0xF5 => self.create_and_execute::<PUSH_AF>(opcode),
            0xC5 => self.create_and_execute::<PUSH_BC>(opcode),
            0xD5 => self.create_and_execute::<PUSH_DE>(opcode),
            0xE5 => self.create_and_execute::<PUSH_HL>(opcode),

            0xF1 => self.create_and_execute::<POP_AF>(opcode),
            0xC1 => self.create_and_execute::<POP_BC>(opcode),
            0xD1 => self.create_and_execute::<POP_DE>(opcode),
            0xE1 => self.create_and_execute::<POP_HL>(opcode),

            0x87 => self.create_and_execute::<ADD_A_A>(opcode),
            0x80 => self.create_and_execute::<ADD_A_B>(opcode),
            0x81 => self.create_and_execute::<ADD_A_C>(opcode),
            0x82 => self.create_and_execute::<ADD_A_D>(opcode),
            0x83 => self.create_and_execute::<ADD_A_E>(opcode),
            0x84 => self.create_and_execute::<ADD_A_H>(opcode),
            0x85 => self.create_and_execute::<ADD_A_L>(opcode),
            0x86 => self.create_and_execute::<ADD_A_xHL>(opcode),
            0xC6 => self.create_and_execute::<ADD_A_N>(opcode),

            0x8F => self.create_and_execute::<ADC_A_A>(opcode),
            0x88 => self.create_and_execute::<ADC_A_B>(opcode),
            0x89 => self.create_and_execute::<ADC_A_C>(opcode),
            0x8A => self.create_and_execute::<ADC_A_D>(opcode),
            0x8B => self.create_and_execute::<ADC_A_E>(opcode),
            0x8C => self.create_and_execute::<ADC_A_H>(opcode),
            0x8D => self.create_and_execute::<ADC_A_L>(opcode),
            0x8E => self.create_and_execute::<ADC_A_xHL>(opcode),
            0xCE => self.create_and_execute::<ADC_A_N>(opcode),

            0x97 => self.create_and_execute::<SUB_A_A>(opcode),
            0x90 => self.create_and_execute::<SUB_A_B>(opcode),
            0x91 => self.create_and_execute::<SUB_A_C>(opcode),
            0x92 => self.create_and_execute::<SUB_A_D>(opcode),
            0x93 => self.create_and_execute::<SUB_A_E>(opcode),
            0x94 => self.create_and_execute::<SUB_A_H>(opcode),
            0x95 => self.create_and_execute::<SUB_A_L>(opcode),
            0x96 => self.create_and_execute::<SUB_A_xHL>(opcode),
            0xD6 => self.create_and_execute::<SUB_A_N>(opcode),

            0x9F => self.create_and_execute::<SBC_A_A>(opcode),
            0x98 => self.create_and_execute::<SBC_A_B>(opcode),
            0x99 => self.create_and_execute::<SBC_A_C>(opcode),
            0x9A => self.create_and_execute::<SBC_A_D>(opcode),
            0x9B => self.create_and_execute::<SBC_A_E>(opcode),
            0x9C => self.create_and_execute::<SBC_A_H>(opcode),
            0x9D => self.create_and_execute::<SBC_A_L>(opcode),
            0x9E => self.create_and_execute::<SBC_A_xHL>(opcode),
            0xDE => self.create_and_execute::<SBC_A_N>(opcode),

            0xA7 => self.create_and_execute::<AND_A_A>(opcode),
            0xA0 => self.create_and_execute::<AND_A_B>(opcode),
            0xA1 => self.create_and_execute::<AND_A_C>(opcode),
            0xA2 => self.create_and_execute::<AND_A_D>(opcode),
            0xA3 => self.create_and_execute::<AND_A_E>(opcode),
            0xA4 => self.create_and_execute::<AND_A_H>(opcode),
            0xA5 => self.create_and_execute::<AND_A_L>(opcode),
            0xA6 => self.create_and_execute::<AND_A_xHL>(opcode),
            0xE6 => self.create_and_execute::<AND_A_N>(opcode),

            0xB7 => self.create_and_execute::<OR_A_A>(opcode),
            0xB0 => self.create_and_execute::<OR_A_B>(opcode),
            0xB1 => self.create_and_execute::<OR_A_C>(opcode),
            0xB2 => self.create_and_execute::<OR_A_D>(opcode),
            0xB3 => self.create_and_execute::<OR_A_E>(opcode),
            0xB4 => self.create_and_execute::<OR_A_H>(opcode),
            0xB5 => self.create_and_execute::<OR_A_L>(opcode),
            0xB6 => self.create_and_execute::<OR_A_xHL>(opcode),
            0xF6 => self.create_and_execute::<OR_A_N>(opcode),

            0xAF => self.create_and_execute::<XOR_A_A>(opcode),
            0xA8 => self.create_and_execute::<XOR_A_B>(opcode),
            0xA9 => self.create_and_execute::<XOR_A_C>(opcode),
            0xAA => self.create_and_execute::<XOR_A_D>(opcode),
            0xAB => self.create_and_execute::<XOR_A_E>(opcode),
            0xAC => self.create_and_execute::<XOR_A_H>(opcode),
            0xAD => self.create_and_execute::<XOR_A_L>(opcode),
            0xAE => self.create_and_execute::<XOR_A_xHL>(opcode),
            0xEE => self.create_and_execute::<XOR_A_N>(opcode),

            0xBF => self.create_and_execute::<CP_A>(opcode),
            0xB8 => self.create_and_execute::<CP_B>(opcode),
            0xB9 => self.create_and_execute::<CP_C>(opcode),
            0xBA => self.create_and_execute::<CP_D>(opcode),
            0xBB => self.create_and_execute::<CP_E>(opcode),
            0xBC => self.create_and_execute::<CP_H>(opcode),
            0xBD => self.create_and_execute::<CP_L>(opcode),
            0xBE => self.create_and_execute::<CP_xHL>(opcode),
            0xFE => self.create_and_execute::<CP_N>(opcode),

            0x3C => self.create_and_execute::<INC_A>(opcode),
            0x04 => self.create_and_execute::<INC_B>(opcode),
            0x0C => self.create_and_execute::<INC_C>(opcode),
            0x14 => self.create_and_execute::<INC_D>(opcode),
            0x1C => self.create_and_execute::<INC_E>(opcode),
            0x24 => self.create_and_execute::<INC_H>(opcode),
            0x2C => self.create_and_execute::<INC_L>(opcode),
            0x34 => self.create_and_execute::<INC_xHL>(opcode),

            0x3D => self.create_and_execute::<DEC_A>(opcode),
            0x05 => self.create_and_execute::<DEC_B>(opcode),
            0x0D => self.create_and_execute::<DEC_C>(opcode),
            0x15 => self.create_and_execute::<DEC_D>(opcode),
            0x1D => self.create_and_execute::<DEC_E>(opcode),
            0x25 => self.create_and_execute::<DEC_H>(opcode),
            0x2D => self.create_and_execute::<DEC_L>(opcode),
            0x35 => self.create_and_execute::<DEC_xHL>(opcode),

            0x09 => self.create_and_execute::<ADD_HL_BC>(opcode),
            0x19 => self.create_and_execute::<ADD_HL_DE>(opcode),
            0x29 => self.create_and_execute::<ADD_HL_HL>(opcode),
            0x39 => self.create_and_execute::<ADD_HL_SP>(opcode),

            0xE8 => self.create_and_execute::<ADD_SP_N>(opcode),

            0x03 => self.create_and_execute::<INC_BC>(opcode),
            0x13 => self.create_and_execute::<INC_DE>(opcode),
            0x23 => self.create_and_execute::<INC_HL>(opcode),
            0x33 => self.create_and_execute::<INC_SP>(opcode),

            0x0B => self.create_and_execute::<DEC_BC>(opcode),
            0x1B => self.create_and_execute::<DEC_DE>(opcode),
            0x2B => self.create_and_execute::<DEC_HL>(opcode),
            0x3B => self.create_and_execute::<DEC_SP>(opcode),

            0x27 => self.create_and_execute::<DAA>(opcode),
            0x2F => self.create_and_execute::<CPL>(opcode),
            0x3F => self.create_and_execute::<CCF>(opcode),
            0x37 => self.create_and_execute::<SCF>(opcode),
            0x00 => self.create_and_execute::<NOP>(opcode),
            0x76 => self.create_and_execute::<HALT>(opcode),
            0x10 if opcode.b2 == 0x00 => self.create_and_execute::<STOP>(opcode),
            0xF3 => self.create_and_execute::<DI>(opcode),
            0xFB => self.create_and_execute::<EI>(opcode),

            0x07 => self.create_and_execute::<RLCA>(opcode),
            0x17 => self.create_and_execute::<RLA>(opcode),
            0x0F => self.create_and_execute::<RRCA>(opcode),
            0x1F => self.create_and_execute::<RRA>(opcode),

            0xC3 => self.create_and_execute::<JP_NN>(opcode),
            0xC2 => self.create_and_execute::<JP_NZ_NN>(opcode),
            0xCA => self.create_and_execute::<JP_Z_NN>(opcode),
            0xD2 => self.create_and_execute::<JP_NC_NN>(opcode),
            0xDA => self.create_and_execute::<JP_C_NN>(opcode),
            0xE9 => self.create_and_execute::<JP_xHL>(opcode),

            0x18 => self.create_and_execute::<JR_N>(opcode),
            0x20 => self.create_and_execute::<JR_NZ_N>(opcode),
            0x28 => self.create_and_execute::<JR_Z_N>(opcode),
            0x30 => self.create_and_execute::<JR_NC_N>(opcode),
            0x38 => self.create_and_execute::<JR_C_N>(opcode),

            0xCB => match opcode.b2 {
                0x37 => self.create_and_execute::<SWAP_A>(opcode),
                0x30 => self.create_and_execute::<SWAP_B>(opcode),
                0x31 => self.create_and_execute::<SWAP_C>(opcode),
                0x32 => self.create_and_execute::<SWAP_D>(opcode),
                0x33 => self.create_and_execute::<SWAP_E>(opcode),
                0x34 => self.create_and_execute::<SWAP_H>(opcode),
                0x35 => self.create_and_execute::<SWAP_L>(opcode),
                0x36 => self.create_and_execute::<SWAP_xHL>(opcode),

                0x07 => self.create_and_execute::<RLC_A>(opcode),
                0x00 => self.create_and_execute::<RLC_B>(opcode),
                0x01 => self.create_and_execute::<RLC_C>(opcode),
                0x02 => self.create_and_execute::<RLC_D>(opcode),
                0x03 => self.create_and_execute::<RLC_E>(opcode),
                0x04 => self.create_and_execute::<RLC_H>(opcode),
                0x05 => self.create_and_execute::<RLC_L>(opcode),
                0x06 => self.create_and_execute::<RLC_xHL>(opcode),

                0x17 => self.create_and_execute::<RL_A>(opcode),
                0x10 => self.create_and_execute::<RL_B>(opcode),
                0x11 => self.create_and_execute::<RL_C>(opcode),
                0x12 => self.create_and_execute::<RL_D>(opcode),
                0x13 => self.create_and_execute::<RL_E>(opcode),
                0x14 => self.create_and_execute::<RL_H>(opcode),
                0x15 => self.create_and_execute::<RL_L>(opcode),
                0x16 => self.create_and_execute::<RL_xHL>(opcode),

                0x0F => self.create_and_execute::<RRC_A>(opcode),
                0x08 => self.create_and_execute::<RRC_B>(opcode),
                0x09 => self.create_and_execute::<RRC_C>(opcode),
                0x0A => self.create_and_execute::<RRC_D>(opcode),
                0x0B => self.create_and_execute::<RRC_E>(opcode),
                0x0C => self.create_and_execute::<RRC_H>(opcode),
                0x0D => self.create_and_execute::<RRC_L>(opcode),
                0x0E => self.create_and_execute::<RRC_xHL>(opcode),

                0x1F => self.create_and_execute::<RR_A>(opcode),
                0x18 => self.create_and_execute::<RR_B>(opcode),
                0x19 => self.create_and_execute::<RR_C>(opcode),
                0x1A => self.create_and_execute::<RR_D>(opcode),
                0x1B => self.create_and_execute::<RR_E>(opcode),
                0x1C => self.create_and_execute::<RR_H>(opcode),
                0x1D => self.create_and_execute::<RR_L>(opcode),
                0x1E => self.create_and_execute::<RR_xHL>(opcode),

                0x27 => self.create_and_execute::<SLA_A>(opcode),
                0x20 => self.create_and_execute::<SLA_B>(opcode),
                0x21 => self.create_and_execute::<SLA_C>(opcode),
                0x22 => self.create_and_execute::<SLA_D>(opcode),
                0x23 => self.create_and_execute::<SLA_E>(opcode),
                0x24 => self.create_and_execute::<SLA_H>(opcode),
                0x25 => self.create_and_execute::<SLA_L>(opcode),
                0x26 => self.create_and_execute::<SLA_xHL>(opcode),

                0x2F => self.create_and_execute::<SRA_A>(opcode),
                0x28 => self.create_and_execute::<SRA_B>(opcode),
                0x29 => self.create_and_execute::<SRA_C>(opcode),
                0x2A => self.create_and_execute::<SRA_D>(opcode),
                0x2B => self.create_and_execute::<SRA_E>(opcode),
                0x2C => self.create_and_execute::<SRA_H>(opcode),
                0x2D => self.create_and_execute::<SRA_L>(opcode),
                0x2E => self.create_and_execute::<SRA_xHL>(opcode),

                0x3F => self.create_and_execute::<SRL_A>(opcode),
                0x38 => self.create_and_execute::<SRL_B>(opcode),
                0x39 => self.create_and_execute::<SRL_C>(opcode),
                0x3A => self.create_and_execute::<SRL_D>(opcode),
                0x3B => self.create_and_execute::<SRL_E>(opcode),
                0x3C => self.create_and_execute::<SRL_H>(opcode),
                0x3D => self.create_and_execute::<SRL_L>(opcode),
                0x3E => self.create_and_execute::<SRL_xHL>(opcode),

                0x47 => self.create_and_execute::<BIT_0_A>(opcode),
                0x40 => self.create_and_execute::<BIT_0_B>(opcode),
                0x41 => self.create_and_execute::<BIT_0_C>(opcode),
                0x42 => self.create_and_execute::<BIT_0_D>(opcode),
                0x43 => self.create_and_execute::<BIT_0_E>(opcode),
                0x44 => self.create_and_execute::<BIT_0_H>(opcode),
                0x45 => self.create_and_execute::<BIT_0_L>(opcode),
                0x46 => self.create_and_execute::<BIT_0_xHL>(opcode),

                0x4F => self.create_and_execute::<BIT_1_A>(opcode),
                0x48 => self.create_and_execute::<BIT_1_B>(opcode),
                0x49 => self.create_and_execute::<BIT_1_C>(opcode),
                0x4A => self.create_and_execute::<BIT_1_D>(opcode),
                0x4B => self.create_and_execute::<BIT_1_E>(opcode),
                0x4C => self.create_and_execute::<BIT_1_H>(opcode),
                0x4D => self.create_and_execute::<BIT_1_L>(opcode),
                0x4E => self.create_and_execute::<BIT_1_xHL>(opcode),

                0x57 => self.create_and_execute::<BIT_2_A>(opcode),
                0x50 => self.create_and_execute::<BIT_2_B>(opcode),
                0x51 => self.create_and_execute::<BIT_2_C>(opcode),
                0x52 => self.create_and_execute::<BIT_2_D>(opcode),
                0x53 => self.create_and_execute::<BIT_2_E>(opcode),
                0x54 => self.create_and_execute::<BIT_2_H>(opcode),
                0x55 => self.create_and_execute::<BIT_2_L>(opcode),
                0x56 => self.create_and_execute::<BIT_2_xHL>(opcode),

                0x5F => self.create_and_execute::<BIT_3_A>(opcode),
                0x58 => self.create_and_execute::<BIT_3_B>(opcode),
                0x59 => self.create_and_execute::<BIT_3_C>(opcode),
                0x5A => self.create_and_execute::<BIT_3_D>(opcode),
                0x5B => self.create_and_execute::<BIT_3_E>(opcode),
                0x5C => self.create_and_execute::<BIT_3_H>(opcode),
                0x5D => self.create_and_execute::<BIT_3_L>(opcode),
                0x5E => self.create_and_execute::<BIT_3_xHL>(opcode),

                0x67 => self.create_and_execute::<BIT_4_A>(opcode),
                0x60 => self.create_and_execute::<BIT_4_B>(opcode),
                0x61 => self.create_and_execute::<BIT_4_C>(opcode),
                0x62 => self.create_and_execute::<BIT_4_D>(opcode),
                0x63 => self.create_and_execute::<BIT_4_E>(opcode),
                0x64 => self.create_and_execute::<BIT_4_H>(opcode),
                0x65 => self.create_and_execute::<BIT_4_L>(opcode),
                0x66 => self.create_and_execute::<BIT_4_xHL>(opcode),

                0x6F => self.create_and_execute::<BIT_5_A>(opcode),
                0x68 => self.create_and_execute::<BIT_5_B>(opcode),
                0x69 => self.create_and_execute::<BIT_5_C>(opcode),
                0x6A => self.create_and_execute::<BIT_5_D>(opcode),
                0x6B => self.create_and_execute::<BIT_5_E>(opcode),
                0x6C => self.create_and_execute::<BIT_5_H>(opcode),
                0x6D => self.create_and_execute::<BIT_5_L>(opcode),
                0x6E => self.create_and_execute::<BIT_5_xHL>(opcode),

                0x77 => self.create_and_execute::<BIT_6_A>(opcode),
                0x70 => self.create_and_execute::<BIT_6_B>(opcode),
                0x71 => self.create_and_execute::<BIT_6_C>(opcode),
                0x72 => self.create_and_execute::<BIT_6_D>(opcode),
                0x73 => self.create_and_execute::<BIT_6_E>(opcode),
                0x74 => self.create_and_execute::<BIT_6_H>(opcode),
                0x75 => self.create_and_execute::<BIT_6_L>(opcode),
                0x76 => self.create_and_execute::<BIT_6_xHL>(opcode),

                0x7F => self.create_and_execute::<BIT_7_A>(opcode),
                0x78 => self.create_and_execute::<BIT_7_B>(opcode),
                0x79 => self.create_and_execute::<BIT_7_C>(opcode),
                0x7A => self.create_and_execute::<BIT_7_D>(opcode),
                0x7B => self.create_and_execute::<BIT_7_E>(opcode),
                0x7C => self.create_and_execute::<BIT_7_H>(opcode),
                0x7D => self.create_and_execute::<BIT_7_L>(opcode),
                0x7E => self.create_and_execute::<BIT_7_xHL>(opcode),

                0xC7 => self.create_and_execute::<SET_0_A>(opcode),
                0xC0 => self.create_and_execute::<SET_0_B>(opcode),
                0xC1 => self.create_and_execute::<SET_0_C>(opcode),
                0xC2 => self.create_and_execute::<SET_0_D>(opcode),
                0xC3 => self.create_and_execute::<SET_0_E>(opcode),
                0xC4 => self.create_and_execute::<SET_0_H>(opcode),
                0xC5 => self.create_and_execute::<SET_0_L>(opcode),
                0xC6 => self.create_and_execute::<SET_0_xHL>(opcode),

                0xCF => self.create_and_execute::<SET_1_A>(opcode),
                0xC8 => self.create_and_execute::<SET_1_B>(opcode),
                0xC9 => self.create_and_execute::<SET_1_C>(opcode),
                0xCA => self.create_and_execute::<SET_1_D>(opcode),
                0xCB => self.create_and_execute::<SET_1_E>(opcode),
                0xCC => self.create_and_execute::<SET_1_H>(opcode),
                0xCD => self.create_and_execute::<SET_1_L>(opcode),
                0xCE => self.create_and_execute::<SET_1_xHL>(opcode),

                0xD7 => self.create_and_execute::<SET_2_A>(opcode),
                0xD0 => self.create_and_execute::<SET_2_B>(opcode),
                0xD1 => self.create_and_execute::<SET_2_C>(opcode),
                0xD2 => self.create_and_execute::<SET_2_D>(opcode),
                0xD3 => self.create_and_execute::<SET_2_E>(opcode),
                0xD4 => self.create_and_execute::<SET_2_H>(opcode),
                0xD5 => self.create_and_execute::<SET_2_L>(opcode),
                0xD6 => self.create_and_execute::<SET_2_xHL>(opcode),

                0xDF => self.create_and_execute::<SET_3_A>(opcode),
                0xD8 => self.create_and_execute::<SET_3_B>(opcode),
                0xD9 => self.create_and_execute::<SET_3_C>(opcode),
                0xDA => self.create_and_execute::<SET_3_D>(opcode),
                0xDB => self.create_and_execute::<SET_3_E>(opcode),
                0xDC => self.create_and_execute::<SET_3_H>(opcode),
                0xDD => self.create_and_execute::<SET_3_L>(opcode),
                0xDE => self.create_and_execute::<SET_3_xHL>(opcode),

                0xE7 => self.create_and_execute::<SET_4_A>(opcode),
                0xE0 => self.create_and_execute::<SET_4_B>(opcode),
                0xE1 => self.create_and_execute::<SET_4_C>(opcode),
                0xE2 => self.create_and_execute::<SET_4_D>(opcode),
                0xE3 => self.create_and_execute::<SET_4_E>(opcode),
                0xE4 => self.create_and_execute::<SET_4_H>(opcode),
                0xE5 => self.create_and_execute::<SET_4_L>(opcode),
                0xE6 => self.create_and_execute::<SET_4_xHL>(opcode),

                0xEF => self.create_and_execute::<SET_5_A>(opcode),
                0xE8 => self.create_and_execute::<SET_5_B>(opcode),
                0xE9 => self.create_and_execute::<SET_5_C>(opcode),
                0xEA => self.create_and_execute::<SET_5_D>(opcode),
                0xEB => self.create_and_execute::<SET_5_E>(opcode),
                0xEC => self.create_and_execute::<SET_5_H>(opcode),
                0xED => self.create_and_execute::<SET_5_L>(opcode),
                0xEE => self.create_and_execute::<SET_5_xHL>(opcode),

                0xF7 => self.create_and_execute::<SET_6_A>(opcode),
                0xF0 => self.create_and_execute::<SET_6_B>(opcode),
                0xF1 => self.create_and_execute::<SET_6_C>(opcode),
                0xF2 => self.create_and_execute::<SET_6_D>(opcode),
                0xF3 => self.create_and_execute::<SET_6_E>(opcode),
                0xF4 => self.create_and_execute::<SET_6_H>(opcode),
                0xF5 => self.create_and_execute::<SET_6_L>(opcode),
                0xF6 => self.create_and_execute::<SET_6_xHL>(opcode),

                0xFF => self.create_and_execute::<SET_7_A>(opcode),
                0xF8 => self.create_and_execute::<SET_7_B>(opcode),
                0xF9 => self.create_and_execute::<SET_7_C>(opcode),
                0xFA => self.create_and_execute::<SET_7_D>(opcode),
                0xFB => self.create_and_execute::<SET_7_E>(opcode),
                0xFC => self.create_and_execute::<SET_7_H>(opcode),
                0xFD => self.create_and_execute::<SET_7_L>(opcode),
                0xFE => self.create_and_execute::<SET_7_xHL>(opcode),

                0x87 => self.create_and_execute::<RES_0_A>(opcode),
                0x80 => self.create_and_execute::<RES_0_B>(opcode),
                0x81 => self.create_and_execute::<RES_0_C>(opcode),
                0x82 => self.create_and_execute::<RES_0_D>(opcode),
                0x83 => self.create_and_execute::<RES_0_E>(opcode),
                0x84 => self.create_and_execute::<RES_0_H>(opcode),
                0x85 => self.create_and_execute::<RES_0_L>(opcode),
                0x86 => self.create_and_execute::<RES_0_xHL>(opcode),

                0x8F => self.create_and_execute::<RES_1_A>(opcode),
                0x88 => self.create_and_execute::<RES_1_B>(opcode),
                0x89 => self.create_and_execute::<RES_1_C>(opcode),
                0x8A => self.create_and_execute::<RES_1_D>(opcode),
                0x8B => self.create_and_execute::<RES_1_E>(opcode),
                0x8C => self.create_and_execute::<RES_1_H>(opcode),
                0x8D => self.create_and_execute::<RES_1_L>(opcode),
                0x8E => self.create_and_execute::<RES_1_xHL>(opcode),

                0x97 => self.create_and_execute::<RES_2_A>(opcode),
                0x90 => self.create_and_execute::<RES_2_B>(opcode),
                0x91 => self.create_and_execute::<RES_2_C>(opcode),
                0x92 => self.create_and_execute::<RES_2_D>(opcode),
                0x93 => self.create_and_execute::<RES_2_E>(opcode),
                0x94 => self.create_and_execute::<RES_2_H>(opcode),
                0x95 => self.create_and_execute::<RES_2_L>(opcode),
                0x96 => self.create_and_execute::<RES_2_xHL>(opcode),

                0x9F => self.create_and_execute::<RES_3_A>(opcode),
                0x98 => self.create_and_execute::<RES_3_B>(opcode),
                0x99 => self.create_and_execute::<RES_3_C>(opcode),
                0x9A => self.create_and_execute::<RES_3_D>(opcode),
                0x9B => self.create_and_execute::<RES_3_E>(opcode),
                0x9C => self.create_and_execute::<RES_3_H>(opcode),
                0x9D => self.create_and_execute::<RES_3_L>(opcode),
                0x9E => self.create_and_execute::<RES_3_xHL>(opcode),

                0xA7 => self.create_and_execute::<RES_4_A>(opcode),
                0xA0 => self.create_and_execute::<RES_4_B>(opcode),
                0xA1 => self.create_and_execute::<RES_4_C>(opcode),
                0xA2 => self.create_and_execute::<RES_4_D>(opcode),
                0xA3 => self.create_and_execute::<RES_4_E>(opcode),
                0xA4 => self.create_and_execute::<RES_4_H>(opcode),
                0xA5 => self.create_and_execute::<RES_4_L>(opcode),
                0xA6 => self.create_and_execute::<RES_4_xHL>(opcode),

                0xAF => self.create_and_execute::<RES_5_A>(opcode),
                0xA8 => self.create_and_execute::<RES_5_B>(opcode),
                0xA9 => self.create_and_execute::<RES_5_C>(opcode),
                0xAA => self.create_and_execute::<RES_5_D>(opcode),
                0xAB => self.create_and_execute::<RES_5_E>(opcode),
                0xAC => self.create_and_execute::<RES_5_H>(opcode),
                0xAD => self.create_and_execute::<RES_5_L>(opcode),
                0xAE => self.create_and_execute::<RES_5_xHL>(opcode),

                0xB7 => self.create_and_execute::<RES_6_A>(opcode),
                0xB0 => self.create_and_execute::<RES_6_B>(opcode),
                0xB1 => self.create_and_execute::<RES_6_C>(opcode),
                0xB2 => self.create_and_execute::<RES_6_D>(opcode),
                0xB3 => self.create_and_execute::<RES_6_E>(opcode),
                0xB4 => self.create_and_execute::<RES_6_H>(opcode),
                0xB5 => self.create_and_execute::<RES_6_L>(opcode),
                0xB6 => self.create_and_execute::<RES_6_xHL>(opcode),

                0xBF => self.create_and_execute::<RES_7_A>(opcode),
                0xB8 => self.create_and_execute::<RES_7_B>(opcode),
                0xB9 => self.create_and_execute::<RES_7_C>(opcode),
                0xBA => self.create_and_execute::<RES_7_D>(opcode),
                0xBB => self.create_and_execute::<RES_7_E>(opcode),
                0xBC => self.create_and_execute::<RES_7_H>(opcode),
                0xBD => self.create_and_execute::<RES_7_L>(opcode),
                0xBE => self.create_and_execute::<RES_7_xHL>(opcode),

                x => panic!("Opcode unknown: 0xCB {:X}", x),
            },

            x => panic!("Opcode unknown: {:X}", x),
        }
    }

    fn create_and_execute<Op: OpConstruct + OpExecute<M>>(&mut self, opcode: Opcode) {
        let op = Op::new(opcode);
        op.execute(&mut self.registers, &mut self.memory);
    }
}

#[derive(Copy, Clone)]
struct Opcode {
    b1: u8,
    b2: u8,
    b3: u8,
}

//    fn get_address(&self) -> u16 {
//        self.code & 0xFFF
//    }

//    fn get_nibble(&self, nibble: u8) -> u8 {
//        let shift = (nibble - 1) * 4;
//        ((self.code & (0xF << shift)) >> shift) as u8
//    }

//    fn get_low_byte(&self) -> u8 {
//        (self.code & 0xFF) as u8
//    }
//}

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
            pc: 0x100,
            sp: 0xFFFE,
            cycles_of_last_command: 0,
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

trait OpExecute<M: Memory> {
    fn execute(&self, registers: &mut Registers, memory: &mut M);
}

fn to_u16(h: u8, l: u8) -> u16 {
    (h as u16) << 8 + l as u16
}

fn store_value_in_register_pair(value: u16, h: &mut u8, l: &mut u8) {
    *h = ((value & 0xFF00) >> 8) as u8;
    *l = (value & 0xFF) as u8;
}

fn decrement_register_pair(h: &mut u8, l: &mut u8) {
    let value = to_u16(*h, *l) - 1;
    store_value_in_register_pair(value, h, l);
}

fn increment_register_pair(h: &mut u8, l: &mut u8) {
    let value = to_u16(*h, *l) + 1;
    store_value_in_register_pair(value, h, l);
}

fn wrapping_add(a: u8, b: u8) -> u8 {
    (Wrapping(a) + Wrapping(b)).0
}

fn wrapping_sub(a: u8, b: u8) -> u8 {
    (Wrapping(a) - Wrapping(b)).0
}

fn wrapping_add_u16(a: u16, b: u16) -> u16 {
    (Wrapping(a) + Wrapping(b)).0
}

fn wrapping_sub_u16(a: u16, b: u16) -> u16 {
    (Wrapping(a) - Wrapping(b)).0
}

macro_rules! create_opcode_struct {
    ($name:ident) => {
        struct $name {
            b1: u8,
            b2: u8,
            b3: u8,
        }

        impl OpConstruct for $name {
            fn new(opcode: Opcode) -> Self {
                $name {
                    b1: opcode.b1,
                    b2: opcode.b2,
                    b3: opcode.b3,
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                registers.$reg = self.b2;
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for LD_xHL_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        memory.write_byte(address, self.b2);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Load (nn) into A where nn is a 16-bit immediate
create_opcode_struct!(LD_A_xNN);
impl<M: Memory> OpExecute<M> for LD_A_xNN {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(self.b3, self.b2);
        registers.a = memory.read_byte(address);
        registers.pc += 3;
        registers.cycles_of_last_command = 16;
    }
}

// Load A into (nn) where nn is a 16-bit immediate
create_opcode_struct!(LD_xNN_A);
impl<M: Memory> OpExecute<M> for LD_xNN_A {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(self.b3, self.b2);
        memory.write_byte(address, registers.a);
        registers.pc += 3;
        registers.cycles_of_last_command = 16;
    }
}

// Load (0xFF00 + C) into A
create_opcode_struct!(LDH_A_xC);
impl<M: Memory> OpExecute<M> for LDH_A_xC {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = 0xFF00 + registers.c as u16;
        registers.a = memory.read_byte(address);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load A into (0xFF00 + C)
create_opcode_struct!(LDH_xC_A);
impl<M: Memory> OpExecute<M> for LDH_xC_A {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = 0xFF00 + registers.c as u16;
        memory.write_byte(address, registers.a);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load (0xFF00 + N) into A
create_opcode_struct!(LDH_A_xN);
impl<M: Memory> OpExecute<M> for LDH_A_xN {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = 0xFF00 + self.b2 as u16;
        registers.a = memory.read_byte(address);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Load A into (0xFF00 + N)
create_opcode_struct!(LDH_xN_A);
impl<M: Memory> OpExecute<M> for LDH_xN_A {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = 0xFF00 + self.b2 as u16;
        memory.write_byte(address, registers.a);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Load (HL) into A. Decrement HL.
create_opcode_struct!(LDD_A_xHL);
impl<M: Memory> OpExecute<M> for LDD_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        registers.a = memory.read_byte(address);
        decrement_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load A into (HL). Decrement HL.
create_opcode_struct!(LDD_xHL_A);
impl<M: Memory> OpExecute<M> for LDD_xHL_A {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        memory.write_byte(address, registers.a);
        decrement_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load (HL) into A. Increment HL.
create_opcode_struct!(LDI_A_xHL);
impl<M: Memory> OpExecute<M> for LDI_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        registers.a = memory.read_byte(address);
        increment_register_pair(&mut registers.h, &mut registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Load A into (HL). Decrement HL.
create_opcode_struct!(LDI_xHL_A);
impl<M: Memory> OpExecute<M> for LDI_xHL_A {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let value = to_u16(self.b3, self.b2);
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
impl<M: Memory> OpExecute<M> for LD_SP_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.sp = to_u16(self.b3, self.b2);
        registers.pc += 3;
        registers.cycles_of_last_command = 12;
    }
}

// Load 16-bit immediate into stack pointer
create_opcode_struct!(LD_SP_HL);
impl<M: Memory> OpExecute<M> for LD_SP_HL {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.sp = to_u16(registers.h, registers.l);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Put SP + n effective address into HL
create_opcode_struct!(LDHL_SP_N);
impl<M: Memory> OpExecute<M> for LDHL_SP_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let address = registers.sp + self.b2 as u16;
        store_value_in_register_pair(address, &mut registers.h, &mut registers.l);
        registers.pc += 2;
        registers.cycles_of_last_command = 12;
    }
}

// Save SP to given address
create_opcode_struct!(LD_xNN_SP);
impl<M: Memory> OpExecute<M> for LD_xNN_SP {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = self.b2 as u16 + (self.b3 as u16) << 8;
        memory.write_word(address, registers.sp);
        registers.pc += 3;
        registers.cycles_of_last_command = 20;
    }
}

// Push register pair onto stack. Decrement SP twice.
macro_rules! push_nn {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
                memory.write_byte(registers.sp, registers.$reg_high);
                memory.write_byte(registers.sp - 1, registers.$reg_low);
                registers.sp -= 2;
                registers.pc += 1;
                registers.cycles_of_last_command = 16;
            }
        }
    )*}
}
push_nn!(
    a, f: PUSH_AF,
    b, c: PUSH_BC,
    d, e: PUSH_DE,
    h, l: PUSH_HL
);

// Pop two bytes off stack into register pair. Increment SP twice.
macro_rules! pop_nn {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
                registers.sp += 2;
                registers.$reg_high = memory.read_byte(registers.sp);
                registers.$reg_low = memory.read_byte(registers.sp - 1);
                registers.pc += 1;
                registers.cycles_of_last_command = 12;
            }
        }
    )*}
}
pop_nn!(
    a, f: POP_AF,
    b, c: POP_BC,
    d, e: POP_DE,
    h, l: POP_HL
);

// Add register to A
macro_rules! add_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let a = registers.a;
                let r = registers.$reg;
                let sum = (Wrapping(a) + Wrapping(r)).0;
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
impl<M: Memory> OpExecute<M> for ADD_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let sum = (Wrapping(a) + Wrapping(val)).0;
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
impl<M: Memory> OpExecute<M> for ADD_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let sum = (Wrapping(a) + Wrapping(self.b2)).0;
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let a = registers.a;
                let r = registers.$reg;
                let carry = registers.get_carry() as u8;
                let sum = (Wrapping(a) + Wrapping(r) + Wrapping(carry)).0;
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
impl<M: Memory> OpExecute<M> for ADC_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = registers.get_carry() as u8;
        let sum = (Wrapping(a) + Wrapping(val) + Wrapping(carry)).0;
        registers.set_zero(sum == 0);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xF) < (a & 0xF));
        registers.set_carry(sum < a);
        registers.a = sum;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(ADC_A_N);
impl<M: Memory> OpExecute<M> for ADC_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let carry = registers.get_carry() as u8;
        let sum = (Wrapping(a) + Wrapping(self.b2) + Wrapping(carry)).0;
        registers.set_zero(sum == 0);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xF) < (a & 0xF));
        registers.set_carry(sum < a);
        registers.a = sum;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Subtract register from A
macro_rules! sub_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let a = registers.a;
                let r = registers.$reg;
                let difference = (Wrapping(a) - Wrapping(r)).0;
                registers.set_zero(difference == 0);
                registers.set_operation(true);
                registers.set_halfcarry((r & 0xF) <= (a & 0xF));
                registers.set_carry(r <= a);
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
impl<M: Memory> OpExecute<M> for SUB_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let difference = (Wrapping(a) - Wrapping(val)).0;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) <= (a & 0xF));
        registers.set_carry(val <= a);
        registers.a = difference;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(SUB_A_N);
impl<M: Memory> OpExecute<M> for SUB_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let val = self.b2;
        let difference = (Wrapping(a) - Wrapping(val)).0;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) <= (a & 0xF));
        registers.set_carry(val <= a);
        registers.a = difference;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Subtract register + carry flag from A
macro_rules! sbc_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let a = registers.a;
                let r = registers.$reg;
                let carry = registers.get_carry() as u8;
                let difference = (Wrapping(a) - Wrapping(r) - Wrapping(carry)).0;
                let r_plus_c = r as u16 + carry as u16;
                registers.set_zero(difference == 0);
                registers.set_operation(true);
                registers.set_halfcarry((r_plus_c & 0xF) < ((a as u16) & 0xF));
                registers.set_carry((r_plus_c) < (a as u16));
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
impl<M: Memory> OpExecute<M> for SBC_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry = registers.get_carry() as u8;
        let difference = (Wrapping(a) - Wrapping(val) - Wrapping(carry)).0;
        let val_plus_c = val as u16 + carry as u16;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val_plus_c & 0xF) < ((a as u16) & 0xF));
        registers.set_carry((val_plus_c) < (a as u16));
        registers.a = difference;
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(SBC_A_N);
impl<M: Memory> OpExecute<M> for SBC_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let val = self.b2;
        let carry = registers.get_carry() as u8;
        let difference = (Wrapping(a) - Wrapping(val) - Wrapping(carry)).0;
        let val_plus_c = val as u16 + carry as u16;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val_plus_c & 0xF) < ((a as u16) & 0xF));
        registers.set_carry((val_plus_c) < (a as u16));
        registers.a = difference;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Store logical AND of register and A in A
macro_rules! and_a_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for AND_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for AND_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let val = self.b2;
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for OR_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for OR_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let val = self.b2;
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for XOR_A_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for XOR_A_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let val = self.b2;
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let a = registers.a;
                let r = registers.$reg;
                let difference = (Wrapping(a) - Wrapping(r)).0;
                registers.set_zero(difference == 0);
                registers.set_operation(true);
                registers.set_halfcarry((r & 0xF) <= (a & 0xF));
                registers.set_carry(r <= a);
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
impl<M: Memory> OpExecute<M> for CP_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let difference = (Wrapping(a) - Wrapping(val)).0;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) <= (a & 0xF));
        registers.set_carry(val <= a);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(CP_N);
impl<M: Memory> OpExecute<M> for CP_N {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let a = registers.a;
        let val = self.b2;
        let difference = (Wrapping(a) - Wrapping(val)).0;
        registers.set_zero(difference == 0);
        registers.set_operation(true);
        registers.set_halfcarry((val & 0xF) <= (a & 0xF));
        registers.set_carry(val <= a);
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}

// Increment register
macro_rules! inc_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let val = registers.$reg;
                let halfcarry = (val & 0xF) == 0xF;
                registers.set_halfcarry(halfcarry);
                let new_val = wrapping_add(val, 1);
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
impl<M: Memory> OpExecute<M> for INC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let halfcarry = (val & 0xF) == 0xF;
        registers.set_halfcarry(halfcarry);
        let new_val = wrapping_add(val, 1);
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let val = registers.$reg;
                let new_val = wrapping_sub(val, 1);
                registers.$reg = new_val;
                let borrow = (new_val & 0xF) == 0xF;
                registers.set_halfcarry(!borrow);
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
impl<M: Memory> OpExecute<M> for DEC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let new_val = wrapping_sub(val, 1);
        memory.write_byte(address, new_val);
        let borrow = (new_val & 0xF) == 0xF;
        registers.set_halfcarry(!borrow);
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let hl = to_u16(registers.h, registers.l);
                let rr = to_u16(registers.$reg_high, registers.$reg_low);
                let sum = wrapping_add_u16(hl, rr);
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
impl<M: Memory> OpExecute<M> for ADD_HL_SP {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let hl = to_u16(registers.h, registers.l);
        let sum = wrapping_add_u16(hl, registers.sp);
        store_value_in_register_pair(sum, &mut registers.h, &mut registers.l);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xFFF) < (hl & 0xFFF));
        registers.set_carry(sum < hl);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

create_opcode_struct!(ADD_SP_N);
impl<M: Memory> OpExecute<M> for ADD_SP_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let sp = registers.sp;
        let sum = wrapping_add_u16(sp, self.b2 as u16);
        registers.sp = sum;
        registers.set_zero(false);
        registers.set_operation(false);
        registers.set_halfcarry((sum & 0xFFF) < (sp & 0xFFF)); // ??? Stimmt das???
        registers.set_carry(sum < sp); // ??? Stimmt das???
        registers.pc += 1;
        registers.cycles_of_last_command = 16;
    }
}

// Increment register pair
macro_rules! inc_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let val = to_u16(registers.$reg_high, registers.$reg_low);
                let new_val = wrapping_add_u16(val, 1);
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
impl<M: Memory> OpExecute<M> for INC_SP {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.sp = wrapping_add_u16(registers.sp, 1);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Decrement register pair
macro_rules! dec_rr {
    ($($reg_high:ident, $reg_low:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
                let val = to_u16(registers.$reg_high, registers.$reg_low);
                let new_val = wrapping_sub_u16(val, 1);
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
impl<M: Memory> OpExecute<M> for DEC_SP {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.sp = wrapping_sub_u16(registers.sp, 1);
        registers.pc += 1;
        registers.cycles_of_last_command = 8;
    }
}

// Swap upper and lower nibbles of register
macro_rules! swap_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for SWAP_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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

// BCD correction for register A
create_opcode_struct!(DAA);
impl<M: Memory> OpExecute<M> for DAA {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let mut val = registers.a;
        if (val & 0xF) > 0x9 || registers.get_halfcarry() {
            val += 0x6;
        }
        if (val & 0xF0) > 0x90 || registers.get_carry() {
            val += 0x60;
            registers.set_carry(true);
        } else {
            registers.set_carry(false);
        }
        registers.a = val;
        registers.set_zero(val == 0);
        registers.set_halfcarry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Complement A register
create_opcode_struct!(CPL);
impl<M: Memory> OpExecute<M> for CPL {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.a ^= 0xFF;
        registers.set_operation(true);
        registers.set_halfcarry(true);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Complement carry flag
create_opcode_struct!(CCF);
impl<M: Memory> OpExecute<M> for CCF {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for SCF {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.set_carry(true);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// No operation
create_opcode_struct!(NOP);
impl<M: Memory> OpExecute<M> for NOP {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Power down CPU until an interrupt occurs
create_opcode_struct!(HALT);
impl<M: Memory> OpExecute<M> for HALT {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        //TODO: Implement
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Halt CPU & LCD display until button pressed
create_opcode_struct!(STOP);
impl<M: Memory> OpExecute<M> for STOP {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        //TODO: Implement
        registers.pc += 2;
        registers.cycles_of_last_command = 4;
    }
}

// Disables interrupts after the next instruction is executed
create_opcode_struct!(DI);
impl<M: Memory> OpExecute<M> for DI {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        //TODO: Implement
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Enables interrupts after the next instruction is executed
create_opcode_struct!(EI);
impl<M: Memory> OpExecute<M> for EI {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        //TODO: Implement
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A left
create_opcode_struct!(RLCA);
impl<M: Memory> OpExecute<M> for RLCA {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let val = registers.a;
        let carry = (val & 0x80) >> 7;
        let new_val = (val << 1) + carry;
        registers.a = new_val;
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry != 0);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A left through carry flag
create_opcode_struct!(RLA);
impl<M: Memory> OpExecute<M> for RLA {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let val = registers.a;
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x80) != 0;
        let new_val = (val << 1) + carry_in;
        registers.a = new_val;
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry_out);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A right
create_opcode_struct!(RRCA);
impl<M: Memory> OpExecute<M> for RRCA {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let val = registers.a;
        let carry = val & 0x1;
        let new_val = (val >> 1) + carry * 0x80;
        registers.a = new_val;
        registers.set_zero(new_val == 0);
        registers.set_operation(false);
        registers.set_halfcarry(false);
        registers.set_carry(carry != 0);
        registers.pc += 1;
        registers.cycles_of_last_command = 4;
    }
}

// Rotate A right through carry flag
create_opcode_struct!(RRA);
impl<M: Memory> OpExecute<M> for RRA {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let val = registers.a;
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x1) != 0;
        let new_val = (val >> 1) + carry_in * 0x80;
        registers.a = new_val;
        registers.set_zero(new_val == 0);
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for RLC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for RL_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for RRC_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for RR_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        let val = memory.read_byte(address);
        let carry_in = registers.get_carry() as u8;
        let carry_out = (val & 0x1) != 0;
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

// Shift register left into carry flag. LSB set to 0.
macro_rules! sla_r {
    ($($reg:ident : $name:ident),*) => {$(
        create_opcode_struct!($name);
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for SLA_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for SRA_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for SRL_xHL {
    fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
                let address = to_u16(registers.h, registers.l);
                let reg = memory.read_byte(address);
                let bit = (0x1 << $bit) & reg != 0;
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, _memory: &mut M) {
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
        impl<M: Memory> OpExecute<M> for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut M) {
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
impl<M: Memory> OpExecute<M> for JP_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let address = to_u16(self.b3, self.b2);
        registers.pc = address;
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if Z flag is reset
create_opcode_struct!(JP_NZ_NN);
impl<M: Memory> OpExecute<M> for JP_NZ_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if !registers.get_zero() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if Z flag is set
create_opcode_struct!(JP_Z_NN);
impl<M: Memory> OpExecute<M> for JP_Z_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if registers.get_zero() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if C flag is reset
create_opcode_struct!(JP_NC_NN);
impl<M: Memory> OpExecute<M> for JP_NC_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if !registers.get_carry() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address nn if C flag is set
create_opcode_struct!(JP_C_NN);
impl<M: Memory> OpExecute<M> for JP_C_NN {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if registers.get_carry() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = address;
        } else {
            registers.pc += 3;
        }
        registers.cycles_of_last_command = 12;
    }
}

// Jump to address in HL
create_opcode_struct!(JP_xHL);
impl<M: Memory> OpExecute<M> for JP_xHL {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        let address = to_u16(registers.h, registers.l);
        registers.pc = address;
        registers.cycles_of_last_command = 4;
    }
}

// Jump to n + current address
create_opcode_struct!(JR_N);
impl<M: Memory> OpExecute<M> for JR_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        registers.pc = ((registers.pc as i32) + self.b2 as i8 as i32) as u16;
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if Z flag is reset
create_opcode_struct!(JR_NZ_N);
impl<M: Memory> OpExecute<M> for JR_NZ_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if !registers.get_zero() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = ((registers.pc as i32) + self.b2 as i8 as i32) as u16;
        } else {
            registers.pc += 2;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if Z flag is set
create_opcode_struct!(JR_Z_N);
impl<M: Memory> OpExecute<M> for JR_Z_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if registers.get_zero() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = ((registers.pc as i32) + self.b2 as i8 as i32) as u16;
        } else {
            registers.pc += 2;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if C flag is reset
create_opcode_struct!(JR_NC_N);
impl<M: Memory> OpExecute<M> for JR_NC_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if !registers.get_carry() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = ((registers.pc as i32) + self.b2 as i8 as i32) as u16;
        } else {
            registers.pc += 2;
        }
        registers.cycles_of_last_command = 8;
    }
}

// Jump to n + current address if C flag is set
create_opcode_struct!(JR_C_N);
impl<M: Memory> OpExecute<M> for JR_C_N {
    fn execute(&self, registers: &mut Registers, _memory: &mut M) {
        if registers.get_carry() {
        let address = to_u16(self.b3, self.b2);
            registers.pc = ((registers.pc as i32) + self.b2 as i8 as i32) as u16;
        } else {
            registers.pc += 2;
        }
        registers.cycles_of_last_command = 8;
    }
}

