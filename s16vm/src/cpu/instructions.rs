pub mod error;
pub mod register;
pub mod word;

use error::InstructionError;
use register::Register as R;
use word::Word;

pub enum Jump {
    Call,          // CALL
    Unconditional, // JMP
    Zero,          // JZ
    NotZero,       // JNZ
    GreaterThan,   // JGT
}

pub enum Instruction {
    Add { rd: R, rs: R, rt: R },
    Sub { rd: R, rs: R, rt: R },
    And { rd: R, rs: R, rt: R },
    Or { rd: R, rs: R, rt: R },
    Xor { rd: R, rs: R, rt: R },
    Not { rd: R, rt: R },
    Sll { rd: R, rs: R, rt: R },
    Shr { rd: R, rs: R, rt: R },
    LoadIndirect { rd: R, rs: R },
    StoreIndirect { rd: R, rs: R },
    Cmp { rs: R, rt: R },
    Return,
    Push { rs: R },
    Pop { rd: R },

    AddImmediate { rt: R, imm: i8 },
    AndImmediate { rt: R, imm: u8 },
    OrImmediate { rt: R, imm: u8 },
    LoadUperImmediate { rt: R, imm: u8 },
    CmpImmediate { rt: R, imm: i8 },
    Load { rt: R, addr: u8 },
    Store { rt: R, addr: u8 },

    Jump { jump_type: Jump, offset: u16 },

    MoveFromSpecial { rt: R, spec: R },
    MoveFromToSpecial { rt: R, spec: R },

    // System operations
    Nop,
    Halt,
    Sysall,
}

type Result<T> = std::result::Result<T, InstructionError>;

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;
        match self {
            Add { rd, rs, rt } => write!(f, "ADD {}, {}, {}", rd, rs, rt),
            Sub { rd, rs, rt } => write!(f, "SUB {}, {}, {}", rd, rs, rt),
            And { rd, rs, rt } => write!(f, "AND {}, {}, {}", rd, rs, rt),
            Or { rd, rs, rt } => write!(f, "OR {}, {}, {}", rd, rs, rt),
            Xor { rd, rs, rt } => write!(f, "XOR {}, {}, {}", rd, rs, rt),
            Not { rd, rt } => write!(f, "NOT {}, {}", rd, rt),
            Sll { rd, rs, rt } => write!(f, "Sll"),
            Shr { rd, rs, rt } => write!(f, "Shr"),
            LoadIndirect { rd, rs } => write!(f, "LoadIndirect"),
            StoreIndirect { rd, rs } => write!(f, "StoreIndirect"),
            Cmp { rs, rt } => write!(f, "Cmp"),
            Return => write!(f, "Return"),
            Push { rs } => write!(f, "Push"),
            Pop { rd } => write!(f, "Pop"),
            AddImmediate { rt, imm } => write!(f, "ADDI {}, {}", rt, imm),
            AndImmediate { rt, imm } => write!(f, "AndImmediate"),
            OrImmediate { rt, imm } => write!(f, "ORI {}, {}", rt, imm),
            LoadUperImmediate { rt, imm } => write!(f, "LoadUperImmediate"),
            CmpImmediate { rt, imm } => write!(f, "CmpImmediate"),
            Load { rt, addr } => write!(f, "LOAD {}, 0x{:04X}", rt, addr),
            Store { rt, addr } => write!(f, "STORE {}, 0x{:04X}", rt, addr),
            Jump { jump_type, offset } => write!(f, "Jump"),
            MoveFromSpecial { rt, spec } => write!(f, "MoveFromSpecial"),
            MoveFromToSpecial { rt, spec } => write!(f, "MoveFromToSpecial"),
            Nop => write!(f, "Nop"),
            Halt => write!(f, "HALT"),
            Sysall => write!(f, "Sysall"),
        }
    }
}

impl Instruction {
    pub fn decode(w: Word) -> Result<Instruction> {
        let instrruction = match w {
            Word::RType {
                opcode,
                rd,
                rs,
                rt,
                funct,
            } => {
                let rd = R::new(rd)?;
                let rs = R::new(rs)?;
                let rt = R::new(rt)?;

                match (opcode, funct) {
                    (0x0, 0x0) => Instruction::Add { rd, rs, rt },
                    (0x0, 0x1) => Instruction::Sub { rd, rs, rt },
                    (0x0, 0x2) => Instruction::And { rd, rs, rt },
                    (0x0, 0x3) => Instruction::Or { rd, rs, rt },
                    (0x0, 0x4) => Instruction::Xor { rd, rs, rt },
                    (0x0, 0x5) => Instruction::Not { rd, rt },
                    (0x0, 0x6) => Instruction::Sll { rd, rs, rt },
                    (0x0, 0x7) => Instruction::Shr { rd, rs, rt },
                    (0x1, 0x0) => Instruction::LoadIndirect { rd, rs },
                    (0x1, 0x1) => Instruction::StoreIndirect { rd, rs },
                    (0x1, 0x2) => Instruction::Cmp { rs, rt },
                    (0x1, 0x3) => Instruction::Return,
                    (0x1, 0x4) => Instruction::Push { rs },
                    (0x1, 0x5) => Instruction::Pop { rd },
                    _ => return Err(InstructionError::InvalidRType(opcode, funct)),
                }
            }

            Word::IType { opcode, rt, imm } => {
                let rt = R::new(rt)?;

                match opcode {
                    0x2 => Instruction::Load { rt, addr: imm },
                    0x3 => Instruction::Store { rt, addr: imm },
                    0x4 => Instruction::AddImmediate { rt, imm: imm as i8 },
                    0x5 => Instruction::AndImmediate { rt, imm },
                    0x6 => Instruction::OrImmediate { rt, imm },
                    0x7 => Instruction::LoadUperImmediate { rt, imm },
                    0x8 => Instruction::CmpImmediate { rt, imm: imm as i8 },
                    _ => return Err(InstructionError::InvalidIType(opcode)),
                }
            }

            Word::JType { opcode, offset } => {
                use Jump::*;
                match opcode {
                    0x9 => Instruction::Jump {
                        jump_type: Call,
                        offset,
                    },
                    0xA => Instruction::Jump {
                        jump_type: Unconditional,
                        offset,
                    },
                    0xB => Instruction::Jump {
                        jump_type: Zero,
                        offset,
                    },
                    0xC => Instruction::Jump {
                        jump_type: NotZero,
                        offset,
                    },
                    0xD => Instruction::Jump {
                        jump_type: GreaterThan,
                        offset,
                    },
                    _ => return Err(InstructionError::InvalidJType(opcode)),
                }
            }

            Word::EType { subcode, rs, rt } => match subcode {
                0x0 => Instruction::Nop,
                0xE => Instruction::Sysall,
                0xF => Instruction::Halt,

                0x1 => {
                    // MOVS Rt, SPEC ; instruction is [0xF][SUB][Rs][Rt][0]
                    let spec = match rt {
                        0 => R::PC,
                        1 => R::SP,
                        2 => R::FLAGS,
                        _ => return Err(InstructionError::InvalidSpecialRegister(rt)),
                    };
                    let rt = R::new(rs)?;
                    Instruction::MoveFromSpecial { rt, spec }
                }
                0x2 => {
                    // MOVS SPEC, Rt ; instruction is [0xF][SUB][Rs][Rt][0]
                    let rt = R::new(rt)?;
                    let spec = match rs {
                        0 => R::PC,
                        1 => R::SP,
                        2 => R::FLAGS,
                        _ => return Err(InstructionError::InvalidSpecialRegister(rs)),
                    };
                    Instruction::MoveFromToSpecial { rt, spec }
                }
                _ => return Err(InstructionError::InvalidEType(subcode)),
            },
        };

        Ok(instrruction)
    }
}
