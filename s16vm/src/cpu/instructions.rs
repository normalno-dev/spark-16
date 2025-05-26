pub mod register;
pub mod word;
pub mod error;

use register::{Register as R, SpecialRegister as SR};
use error::InstructionError;
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

    AddImmediate { rt: R, imm: u8 },
    AndImmediate { rt: R, imm: u8 },
    OrImmediate { rt: R, imm: u8 },
    LoadUperImmediate { rt: R, imm: u8 },
    CmpImmediate { rt: R, imm: u8 },
    Load { rt: R, addr: u8 },
    Store { rt: R, addr: u8 },

    Jump { jump_type: Jump, offset: i16 },

    MoveFromSpecialToReg { rt: R, spec: SR },
    MoveFromRegToSpecial { rt: R, spec: SR },

    // System operations
    Nop,
    Halt,
    Sysall,

    // Special invalid instruction
    ERR(String),
}

type Result<T> = std::result::Result<T, InstructionError>;

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
                    _ => return Err(InstructionError::InvalidRType(opcode, funct))
                }
            }

            Word::IType { opcode, rt, imm } => {
                let rt = R::new(rt)?;

                match opcode {
                    0x0 => Instruction::Load { rt, addr: imm },
                    0x1 => Instruction::Store { rt, addr: imm },
                    0x2 => Instruction::AddImmediate { rt, imm },
                    0x3 => Instruction::AndImmediate { rt, imm },
                    0x4 => Instruction::OrImmediate { rt, imm },
                    0x5 => Instruction::LoadUperImmediate { rt, imm },
                    0x6 => Instruction::CmpImmediate { rt, imm },
                    _ => return Err(InstructionError::InvalidIType(opcode)),
                }
            }

            Word::JType { opcode, offset } => {
                use Jump::*;
                match opcode {
                    0x7 => Instruction::Jump {jump_type: Call, offset},
                    0x8 => Instruction::Jump {jump_type: Unconditional, offset},
                    0x9 => Instruction::Jump {jump_type: Zero, offset},
                    0xA => Instruction::Jump {jump_type: NotZero, offset},
                    0xB => Instruction::Jump {jump_type: GreaterThan, offset},
                    _ => return Err(InstructionError::InvalidJType(opcode)),
                }
            }

            Word::EType { subcode, rs, rt } => match subcode {
                0x0 => Instruction::Nop,
                0xE => Instruction::Sysall,
                0xF => Instruction::Halt,
                
                0x1 => { // MOVS Rt, SPEC ; instruction is [0xF][SUB][Rs][Rt][0]
                    let spec = SR::new(rt)?;
                    let rt = R::new(rs)?;
                    Instruction::MoveFromSpecialToReg { rt, spec }
                }
                0x2 => { // MOVS SPEC, Rt ; instruction is [0xF][SUB][Rs][Rt][0]
                    let rt = R::new(rt)?;
                    let spec = SR::new(rs)?;
                    Instruction::MoveFromRegToSpecial { rt, spec }
                }
                _ => return Err(InstructionError::InvalidEType(subcode)),
            },
        };

        Ok(instrruction)
    }
}
