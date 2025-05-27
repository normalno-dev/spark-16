mod instructions;
mod error;
mod memory;
mod types;

use error::CpuError;
use instructions::{register::Register, word::Word, Instruction, Jump};
use memory::Memory;

type Result<T> = std::result::Result<T, CpuError>;

#[derive(Default)]
struct Flags {
    zero: bool,
    negative: bool,
    carry: bool,
    overflow: bool,
}

impl Flags {
    fn as_u16(&self) -> u16 {
        (self.zero as u16)     << 0 |
        (self.carry as u16)    << 1 |
        (self.negative as u16) << 2 |
        (self.overflow as u16) << 3
    }

    fn from_u16(&mut self, value: u16) {
        self.zero     = (value & 0x01) != 0;
        self.carry    = (value & 0x02) != 0;
        self.negative = (value & 0x04) != 0;
        self.overflow = (value & 0x08) != 0;
    }
}

pub struct CPU {
    registers: [u16; 8], // 8 general-purpose registers R0-R7
    pc: u16,             // Program Counter
    sp: u16,             // Stack Pointer
    flags: Flags,        // CPU Flags (Z, C, N, V)

    memory: Memory,
    
    halted: bool,

    // used to control program bounderies
    program_start: u16,
    program_end: u16,
}

impl CPU {
    pub fn execute(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Add { rd, rs, rt } => self.op_add(rd,rs,rt),
            Instruction::Sub { rd, rs, rt } => self.op_sub(rd, rs, rt),
            
            Instruction::And { rd, rs, rt } => self.op_logical(rd, rs, rt, LogicalOperation::And),
            Instruction::Or { rd, rs, rt }  => self.op_logical(rd, rs, rt, LogicalOperation::Or),
            Instruction::Xor { rd, rs, rt } => self.op_logical(rd, rs, rt, LogicalOperation::Xor),
            Instruction::Not { rd, rt }               => self.op_logical(rd, Register::R0, rt, LogicalOperation::Not),

            Instruction::Sll { rd, rs, rt } => self.op_shift(rd, rs, rt, ShiftOperation::Left),
            Instruction::Shr { rd, rs, rt } => self.op_shift(rd, rs, rt, ShiftOperation::Right),

            Instruction::LoadIndirect { rd, rs } => self.op_load_indirect(rd, rs),
            Instruction::StoreIndirect { rd, rs } => self.op_store_indirect(rd, rs),
            
            Instruction::Push { rs } => self.op_push(rs),
            Instruction::Pop { rd } => self.op_pop(rd),

            Instruction::Cmp { rs, rt } => self.op_cmp(rs, rt),
            Instruction::Jump { jump_type, offset } => self.op_jump(jump_type, offset),
            Instruction::Return => self.op_ret(),

            Instruction::AddImmediate { rt, imm } => self.op_add_immediate(rt, imm),
            Instruction::AndImmediate { rt, imm } => self.op_and_immediate(rt, imm),
            Instruction::OrImmediate { rt, imm } => self.op_or_immediate(rt, imm),
            Instruction::CmpImmediate { rt, imm } => self.op_cmp_immediate(rt, imm),
            Instruction::LoadUperImmediate { rt, imm } => self.op_load_upper_immediate(rt, imm),

            Instruction::Load { rt, addr } => self.op_load(rt, addr),
            Instruction::Store { rt, addr } => self.op_store(rt, addr),
            
            Instruction::MoveFromSpecial { rt, spec } => self.op_movs(rt, spec, false),
            Instruction::MoveFromToSpecial { rt, spec } => self.op_movs(rt, spec, true),
            
            Instruction::Nop => Ok(()),
            Instruction::Halt => self.op_halt(),
            Instruction::Sysall => Err(CpuError::NotImplementedYet),
        }
    }

    pub fn step(&mut self) -> Result<bool> {
        if self.halted {
            return Ok(false)
        }

        // Security control
        self.secure_boundaries()?;

        // Fetch
        let instruction_word = self.memory.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        // Decode
        let word = Word::new(instruction_word);
        let instruction = match Instruction::decode(word) {
            Ok(x) => x,
            Err(err) => return Err(CpuError::InvalidInstruction(word, err)),
        };

        // Exec
        self.execute(instruction)?;

        Ok(true)
    }

    // Load a program into memory with starting address start_addr.
    // Automatically sets program boundaries and pc.
    pub fn load_program(&mut self, program: &[u8], start_addr: u16) -> Result<()> {
        let program_size = program.len() as u16;

        self.program_start = start_addr;
        self.program_end = start_addr + program_size;
        self.pc = start_addr;
        self.sp = 0xFFFE;

        for (i, &byte) in program.iter().enumerate() {
            let addr = start_addr + i as u16;
            self.memory.write_byte(addr, byte)?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.step()? {};

        Ok(())
    }

    // Control program boundries, it's the simplest way to not fuck up.
    // Later, it should be upgraded to hybrid system based on memory segments and CPU security polices.
    fn secure_boundaries(&self) -> Result<()> {
        // Check if we can read full instruction and not became out of program boundaries.
        let instruction_end = self.pc.saturating_add(2);
        if self.pc < self.program_start || instruction_end > self.program_end {
            Err(CpuError::ProgramBoundsViolation { pc: self.pc, iend: instruction_end, low: self.program_start, high: self.program_end })
        } else {
            Ok(())
        }
    }

    fn get_register(&self, reg: Register) -> u16 {
        use Register::*;
        match reg {
            R0 => 0,
            R1 | R2 | R3 | R4 | R5 | R6 | R7 => self.registers[reg as usize],
            SP => self.sp,
            PC => self.pc,
            FLAGS => self.flags.as_u16(),
        }
    }

    fn set_register(&mut self, reg: Register, val: u16) {
        use Register::*;

        match reg {
            R0 => return, // R0 can't be changed
            R1 | R2 | R3 | R4 | R5 | R6 | R7 => self.registers[reg as usize] = val,
            SP => self.sp = val,
            PC => self.pc = val,
            FLAGS => self.flags.from_u16(val),
        }
    }

    fn update_flags_arithmetic(&mut self, operand_a: u16, operand_b: u16, result: u16, carry: bool, is_substraction: bool) {
        self.flags.zero = result == 0;
        self.flags.negative = (result & 0x8000) != 0;
        self.flags.carry = carry;
        self.flags.overflow = self.check_overflow(operand_a, operand_b, result, is_substraction);
    }

    fn update_flags_logical(&mut self, result: u16) {
        self.flags.zero = result == 0;
        self.flags.negative = (result & 0x8000) != 0;
        self.flags.carry = false;
        self.flags.overflow = false;
    }

    fn update_flags_shift(&mut self, result: u16, last_shifted_bit: bool) {
        self.flags.zero = result == 0;
        self.flags.negative = (result & 0x8000) != 0;
        self.flags.carry = last_shifted_bit;
        self.flags.overflow = false;
    }

    fn check_overflow(&self, operand_a: u16, operand_b: u16, result: u16, is_substraction: bool) -> bool {
        let a_sign = (operand_a & 0x8000) != 0;
        let b_sign = (operand_b & 0x8000) != 0;
        let result_sign = (result & 0x8000) != 0;

        if is_substraction {
            // overflow happens when:
            // 1. positive - negative = negative
            // 2. negative - positive = positive
            (!a_sign && b_sign && result_sign) ||
            (a_sign && !b_sign && !result_sign)
        } else {
            // overflow happens when:
            // 1. negative + negative = positive
            // 2. positive + positive = negative
            (!a_sign && !b_sign && result_sign) ||
            (a_sign && b_sign && !result_sign)
        }
    }

}

enum LogicalOperation {
    And, Or, Xor, Not,
}

enum ShiftOperation {
    Left, Right
}

// Debug methods
impl CPU {
    pub fn get_registers(&self) -> &[u16] {
        &self.registers
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn get_flags(&self) -> &Flags {
        &self.flags
    }

    pub fn get_memory(&self) -> &Memory {
        &self.memory
    }

    pub fn dump_registers(&self) -> String {
        let mut output = String::new();

        for (i, val) in self.registers.iter().enumerate() {
            output.push_str(&format!("R{} 0x{:04X} 0b{:016b} ({:5})\n", i, val, val, val));
        }

        output.push_str(&format!("FLAGS: 0x{:04X} [Z:{} C:{} N:{} V:{}]\n", 
            self.flags.as_u16(), 
            self.flags.zero,
            self.flags.carry,
            self.flags.negative,
            self.flags.overflow,
        ));

        output
    }

    pub fn dump_memory_hex(&self, start: u16, length: u16) -> String {
        let mut output = String::new();
        let data = self.memory.get_range(start, length);

        for (i, byte) in data.iter().enumerate() {
            if i % 16 == 0 {
                output.push_str(&format!("0x{:04X}: ", start + i as u16));
            }

            output.push_str(&format!("0x{:2X} ", byte));
            if i % 16 == 15 {
                output.push('\n');
            }
        }
        
        output
    }
}

// Instruction implementations
impl CPU {
    fn op_add(&mut self, rd: Register, rs: Register, rt: Register) -> Result<()> {
        let a = self.get_register(rs);
        let b = self.get_register(rt);
        let (result, carry) = a.overflowing_add(b);

        self.set_register(rd, result);
        self.update_flags_arithmetic(a, b, result, carry, false);

        Ok(())
    }

    fn op_sub(&mut self, rd: Register, rs: Register, rt: Register) -> Result<()> {
        let a = self.get_register(rs);
        let b = self.get_register(rt);
        let (result, borrow) = a.overflowing_sub(b);

        self.set_register(rd, result);
        self.update_flags_arithmetic(a, b, result, borrow, true);

        Ok(())
    }

    fn op_logical(&mut self, rd: Register, rs: Register, rt: Register, op: LogicalOperation) -> Result<()> {
        let a = self.get_register(rs);
        let b = self.get_register(rt);
        let result = match op {
            LogicalOperation::And => a & b,
            LogicalOperation::Or  => a | b,
            LogicalOperation::Xor => a ^ b,
            LogicalOperation::Not => !a,
        };

        self.set_register(rd, result);
        self.update_flags_logical(result);

        Ok(())
    }

    fn op_shift(&mut self, rd: Register, rs:Register, rt: Register, op: ShiftOperation) -> Result<()> {
        let value = self.get_register(rs);
        let s_size = self.get_register(rt) & 0xF; // Mask to 4 bits (0-15 range)

        if s_size == 0 {
            self.set_register(rd, value);
            return Ok(());
        }

        let (result, last_shifted_bit) = match op {
            ShiftOperation::Left => {
                (
                    value << s_size,
                    ((value >> (16 - s_size)) & 1 == 1) // when shift to the left, a bigger bit falls out
                )
            }
            ShiftOperation::Right => {
                (
                    value >> s_size,
                    ((value >> (s_size - 1)) & 1 == 1) // when shift to the left, a lower bit falls out
                )
            }
        };

        self.set_register(rd, result);
        self.update_flags_shift(result, last_shifted_bit);

        Ok(())
    }

    fn op_halt(&mut self) -> Result<()> {
        self.halted = true;
        Ok(())
    }

    fn op_load_indirect(&mut self, rd: Register, rs: Register) -> Result<()> {
        let value = self.memory.read_word(rs as u16)?;
        self.set_register(rd, value);
        Ok(())
    }

    fn op_store_indirect(&mut self, rd: Register, rs: Register) -> Result<()> {
        let value = self.get_register(rd);
        self.memory.write_word(rs as u16, value)?;
        Ok(())
    }

    fn op_cmp(&mut self, rs: Register, rt: Register) -> Result<()> {
        let operand_a = self.get_register(rs);
        let operand_b = self.get_register(rt);
        let (result, borrow) = operand_a.overflowing_sub(operand_b);

        self.update_flags_arithmetic(operand_a, operand_b, result, borrow, true);

        Ok(())
    }

    fn op_push(&mut self, rs: Register) -> Result<()> {
        let value = self.get_register(rs);
        if self.sp < 2 {
            return Err(CpuError::StackOverflow)
        }

        self.sp = self.sp.wrapping_sub(2);
        self.memory.write_word(self.sp, value)?;

        Ok(())
    }

    fn op_pop(&mut self, rd: Register) -> Result<()> {
        if self.sp > 0xFFFE {
            return Err(CpuError::StackOverflow)
        }

        let value = self.memory.read_word(self.sp)?;
        self.set_register(rd, value);
        self.sp = self.sp.wrapping_add(2);

        Ok(())
    }

    fn op_ret(&mut self) -> Result<()> {
        self.op_pop(Register::PC)?;

        Ok(())
    }

    // Remember offset is 12 bits!
    fn op_jump(&mut self, jt: Jump, offset: u16) -> Result<()> {
        let signed_offset = types::convert_12bit_to_signed(offset);

        match jt {
            Jump::Call => {
                self.op_push(Register::PC)?;
                self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
            },
            Jump::Unconditional => {
                self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
            },
            Jump::Zero => {
                if self.flags.zero {
                    self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
                }
            },
            Jump::NotZero => {
                if !self.flags.zero {
                    self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
                }
            },
            Jump::GreaterThan => {
                if !self.flags.zero && self.flags.negative == self.flags.overflow {
                    self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
                }
            },
        }

        Ok(())
    }

    fn op_add_immediate(&mut self, rt: Register, imm: i8) -> Result<()> {
        let operand_a = self.get_register(rt);
        let operand_b = imm as u16;
        let (result, carry) = operand_a.overflowing_add(operand_b);

        self.set_register(rt, result);
        self.update_flags_arithmetic(operand_a, operand_b, result, carry, false);
        
        Ok(())
    }

    fn op_and_immediate(&mut self, rt: Register, imm: u8) -> Result<()> {
        let value = self.get_register(rt);
        let imm = imm as u16;
        let result = value & imm;
        
        self.set_register(rt, result);
        self.update_flags_logical(result);
        
        Ok(())
    }

    fn op_or_immediate(&mut self, rt: Register, imm: u8) -> Result<()> {
        let value = self.get_register(rt);
        let imm = imm as u16;
        let result = value & imm;
        
        self.set_register(rt, result);
        self.update_flags_logical(result);
        
        Ok(())
    }

    fn op_cmp_immediate(&mut self, rt: Register, imm: i8) -> Result<()> {
        let operand_a = self.get_register(rt);
        let operand_b = imm as u16;
        let (result, borrow) = operand_a.overflowing_sub(operand_b);

        self.update_flags_arithmetic(operand_a, operand_b, result, borrow, true);

        Ok(())
    }

    // imm is used as 8 bits value
    fn op_load_upper_immediate(&mut self, rt: Register, imm: u8) -> Result<()> {
        let value= (imm as u16) << 8;
        self.set_register(rt, value);

        Ok(())
    }

    fn op_load(&mut self, rt: Register, addr: u8) -> Result<()> {
        let value = self.memory.read_word(addr as u16)?;
        self.set_register(rt, value);

        Ok(())
    }

    fn op_store(&mut self, rt: Register, addr: u8) -> Result<()> {
        let value = self.get_register(rt);
        self.memory.write_word(addr as u16, value)?;

        Ok(())
    }

    fn op_movs(&mut self, rt: Register, spec: Register, to_special: bool) -> Result<()> {
        let (source, target) = match to_special {
            true => (rt, spec),
            false => (spec, rt),
        };

        let value = self.get_register(source);
        self.set_register(target, value);

        Ok(())
    }
}
