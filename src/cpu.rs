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
                registers.set_carry((sum & 0xFF) < (a & 0xFF));
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
        registers.set_carry((sum & 0xFF) < (a & 0xFF));
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
        registers.set_carry((sum & 0xFF) < (a & 0xFF));
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
                registers.set_carry((sum & 0xFF) < (a & 0xFF));
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
        registers.set_carry((sum & 0xFF) < (a & 0xFF));
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
        registers.set_carry((sum & 0xFF) < (a & 0xFF));
        registers.a = sum;
        registers.pc += 2;
        registers.cycles_of_last_command = 8;
    }
}




