mod instructions;
mod error;
mod memory;

use error::CpuError;
use instructions::{register::Register, word::Word, Instruction};
use memory::Memory;

type Result<T> = std::result::Result<T, CpuError>;

struct Flags {
    zero: bool,
    negative: bool,
    carry: bool,
    overflow: bool,
}

impl Flags {
    fn as_u16(self) -> u16 {
        (self.zero as u16)     << 0 |
        (self.carry as u16)    << 1 |
        (self.negative as u16) << 2 |
        (self.overflow as u16) << 3
    }
}

pub struct S16VM {
    registers: [u16; 8], // 8 general-purpose registers R0-R7
    pc: u16,             // Program Counter
    sp: u16,             // Stack Pointer
    flags: Flags,        // CPU Flags (Z, C, N, V)

    memory: Memory, // 64KB of memory
    
    halted: bool,
}

impl S16VM {
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

            Instruction::LoadIndirect { rd, rs } => Err(CpuError::NotImplementedYet),
            Instruction::StoreIndirect { rd, rs } => Err(CpuError::NotImplementedYet),

            Instruction::Cmp { rs, rt } => Err(CpuError::NotImplementedYet),
            Instruction::Return => Err(CpuError::NotImplementedYet),
            Instruction::Push { rs } => Err(CpuError::NotImplementedYet),
            Instruction::Pop { rd } => Err(CpuError::NotImplementedYet),
            Instruction::AddImmediate { rt, imm } => Err(CpuError::NotImplementedYet),
            Instruction::AndImmediate { rt, imm } => Err(CpuError::NotImplementedYet),
            Instruction::OrImmediate { rt, imm } => Err(CpuError::NotImplementedYet),
            Instruction::LoadUperImmediate { rt, imm } => Err(CpuError::NotImplementedYet),
            Instruction::CmpImmediate { rt, imm } => Err(CpuError::NotImplementedYet),
            Instruction::Load { rt, addr } => Err(CpuError::NotImplementedYet),
            Instruction::Store { rt, addr } => Err(CpuError::NotImplementedYet),
            Instruction::Jump { jump_type, offset } => Err(CpuError::NotImplementedYet),
            Instruction::MoveFromSpecialToReg { rt, spec } => Err(CpuError::NotImplementedYet),
            Instruction::MoveFromRegToSpecial { rt, spec } => Err(CpuError::NotImplementedYet),
            Instruction::Nop => Ok(()),
            Instruction::Halt => Err(CpuError::NotImplementedYet),
            Instruction::Sysall => Err(CpuError::NotImplementedYet),
            Instruction::ERR(_) => Err(CpuError::NotImplementedYet),
        }
    }

    pub fn step(&mut self) -> Result<bool> {
        if self.halted {
            return Ok(false)
        }

        let instruction_word = self.memory.read_word(self.pc)?;
        let word = Word::new(instruction_word);

        Ok(true)
    }

    pub fn load() {
        
    }

    fn get_register(&self, reg: Register) -> Result<u16> {
        match reg.idx() {
            0..=7 => Ok(self.registers[reg as usize]),
            _ => Err(CpuError::InvalidRegister(reg)),
        }
    }

    fn set_register(&mut self, reg: Register, val: u16) -> Result<()> {
        match reg.idx() {
            0 => Ok(()), // can't be changed
            1..=7 => {
                self.registers[reg as usize] = val;
                Ok(())
            },
            _ => Err(CpuError::InvalidRegister(reg)),
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

impl S16VM {
    fn op_add(&mut self, rd: Register, rs: Register, rt: Register) -> Result<()> {
        let a = self.get_register(rs)?;
        let b = self.get_register(rt)?;
        let (result, carry) = a.overflowing_add(b);

        self.set_register(rd, result)?;
        self.update_flags_arithmetic(a, b, result, carry, false);

        Ok(())
    }

    fn op_sub(&mut self, rd: Register, rs: Register, rt: Register) -> Result<()> {
        let a = self.get_register(rs)?;
        let b = self.get_register(rt)?;
        let (result, borrow) = a.overflowing_sub(b);

        self.set_register(rd, result)?;
        self.update_flags_arithmetic(a, b, result, borrow, true);

        Ok(())
    }

    fn op_logical(&mut self, rd: Register, rs: Register, rt: Register, op: LogicalOperation) -> Result<()> {
        let a = self.get_register(rs)?;
        let b = self.get_register(rt)?;
        let result = match op {
            LogicalOperation::And => a & b,
            LogicalOperation::Or  => a | b,
            LogicalOperation::Xor => a ^ b,
            LogicalOperation::Not => !a,
        };

        self.set_register(rd, result)?;
        self.update_flags_logical(result);

        Ok(())
    }

    fn op_shift(&mut self, rd: Register, rs:Register, rt: Register, op: ShiftOperation) -> Result<()> {
        let value = self.get_register(rs)?;
        let s_size = self.get_register(rt)? & 0xF; // Mask to 4 bits (0-15 range)

        if s_size == 0 {
            self.set_register(rd, value)?;
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

        self.set_register(rd, result)?;
        self.update_flags_shift(result, last_shifted_bit);

        Ok(())
    }
}