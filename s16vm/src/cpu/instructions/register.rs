use super::error::InstructionError;

type Result<T> = std::result::Result<T, InstructionError>;

#[derive(Debug,Clone, Copy)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,

    // Special Registers
    SP,
    PC,
    FLAGS
}

impl Register {
    pub fn idx(self) -> u8 {
        match self {
            Self::SP | Self::PC | Self::FLAGS => 0xF as u8,
            _ => self as u8,
        }
    }

    pub fn new(id: u8) -> Result<Self> {
        use Register::*;

        let reg = match id {
            0 => R0,
            1 => R1,
            2 => R2,
            3 => R3,
            4 => R4,
            5 => R5,
            6 => R6,
            7 => R7,
            _ => return Err(InstructionError::InvalidRegister(id)),
        };

        Ok(reg)
    }
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        self.idx()
    }
}