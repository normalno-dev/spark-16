use super::{instructions::{self, error::InstructionError, register::{Register}}, memory::MemoryError};
use instructions::word::Word;

#[derive(Debug)]
pub enum CpuError {
    InvalidInstruction(Word, InstructionError),
    MemoryOutOfBounds(MemoryError),
    ProgramBoundsViolation{pc:u16, iend: u16, low: u16, high: u16},
    StackOverflow,
    NotImplementedYet,
}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuError::InvalidInstruction(w, err) => write!(f, "invalid instruction 0x{:X}: {}", w, err),
            CpuError::MemoryOutOfBounds(MemoryError::OutOfBounds(addr)) => write!(f, "memory out of bounds addr=0x{:X}", addr),
            CpuError::StackOverflow => write!(f, "stack overflow"),
            CpuError::NotImplementedYet => write!(f, "instruction is not implemented yet"),
            CpuError::ProgramBoundsViolation { pc, iend, low, high } => 
                write!(f, "PC violation: 0x{:04X} (instruction ends at {:04X}) outside program boundaries [{:04X}, {:04X}]", pc, iend, low, high),
        }
    }
}

impl From<MemoryError> for CpuError {
    fn from(err: MemoryError) -> Self {
        match err {
            MemoryError::OutOfBounds(_) => Self::MemoryOutOfBounds(err)
        }
    }
}

impl std::error::Error for CpuError {}
