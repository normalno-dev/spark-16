use super::word::Word;

#[derive(Debug)]
pub enum InstructionError {
    InvalidRType(u8, u8),
    InvalidIType(u8),
    InvalidJType(u8),
    InvalidEType(u8),

    InvalidRegister(u8),
    InvalidSpecialRegister(u8),
}

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionError::InvalidRType(op, funct) => {
                write!(f, "invalid r-type OP=0x{:X} FUNCT=0x{:X}", op, funct)
            }
            InstructionError::InvalidIType(op) => write!(f, "invalid i-type OP=0x{:X}", op),
            InstructionError::InvalidJType(op) => write!(f, "invalid j-type OP=0x{:X}", op),
            InstructionError::InvalidEType(sub) => write!(f, "invalid e-type OP=0x{:X}", sub),
            InstructionError::InvalidRegister(idx) => {
                write!(f, "register must be 0-7, given {}", idx)
            }
            InstructionError::InvalidSpecialRegister(idx) => {
                write!(f, "special register must be 0-2, given {}", idx)
            }
        }
    }
}

impl std::error::Error for InstructionError {}
