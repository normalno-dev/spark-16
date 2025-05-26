use std::fmt::write;

use super::instructions::{self, error::InstructionError, register::{Register, SpecialRegister}};
use instructions::word::Word;

#[derive(Debug)]
pub enum CpuError {
    InvalidInstruction(Word, InstructionError),
    InvalidRegister(Register),
    InvalidSpecialRegister(SpecialRegister),
    MemoryOutOfBounds,
    StackOverflow,
    NotImplementedYet,
}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuError::InvalidInstruction(w, err) => write!(f, "invalid instruction 0x{:X}: {}", w, err),
            CpuError::MemoryOutOfBounds => write!(f, "memory out of bounds"),
            CpuError::StackOverflow => write!(f, "stack overflow"),
            CpuError::NotImplementedYet => write!(f, "instruction is not implemented yet"),
            CpuError::InvalidRegister(reg) => write!(f, "invalid register {}", reg.idx()),
            CpuError::InvalidSpecialRegister(reg) => write!(f, "invalid special register {}", reg.idx()),
        }
    }
}

impl std::error::Error for CpuError {}
