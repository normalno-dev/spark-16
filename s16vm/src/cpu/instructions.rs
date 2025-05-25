mod register;
mod word;

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
    Add { rd: u8, rs: u8, rt: u8 },
    Sub { rd: u8, rs: u8, rt: u8 },
    And { rd: u8, rs: u8, rt: u8 },
    Or { rd: u8, rs: u8, rt: u8 },
    Xor { rd: u8, rs: u8, rt: u8 },
    Not { rd: u8, rt: u8 },
    Sll { rd: u8, rs: u8, rt: u8 },
    Shr { rd: u8, rs: u8, rt: u8 },
    LoadIndirect { rd: u8, rs: u8 },
    StoreIndirect { rd: u8, rs: u8 },
    Cmp { rs: u8, rt: u8 },
    Return,
    Push { rs: u8 },
    Pop { rd: u8 },

    AddImmediate { rt: u8, imm: u8 },
    AndImmediate { rt: u8, imm: u8 },
    OrImmediate { rt: u8, imm: u8 },
    LoadUperImmediate { rt: u8, imm: u8 },
    CmpImmediate { rt: u8, imm: u8 },
    Load { rt: u8, addr: u8 },
    Store { rt: u8, addr: u8 },

    Jump { jump_type: Jump, offset: i16 },

    // Rmovs: RT = Spec
    RMovs { rt: u8, spec: u8 },
    // WMovs: Spec = RT
    WMovs { rt: u8, spec: u8 },

    // System operations
    Nop,
    Halt,
    Sysall,

    // Special invalid instruction
    ERR(String),
}

impl Instruction {
    pub fn decode(w: Word) -> Result<Instruction, String> {
        fn validate_reg(reg: u8) -> Result<(), String> {
            if reg > 0x7 {
                Err(format!("register must be 0..=7"))
            } else {
                Ok(())
            }
        }

        fn validate_spec(spec: u8) -> Result<(), String> {
            if spec > 0x3 {
                Err(format!("SPEC must be 0..=7"))
            } else {
                Ok(())
            }
        }

        let instrruction = match w {
            Word::RType {
                opcode,
                rd,
                rs,
                rt,
                funct,
            } => {
                validate_reg(rd)?;
                validate_reg(rs)?;
                validate_reg(rt)?;

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
                    _ => {
                        return Err(format!(
                            "invalid R-type instruction: OP=0x{:X} FUNCT=0x{:X}",
                            opcode, funct
                        ))
                    }
                }
            }

            Word::IType { opcode, rt, imm } => {
                validate_reg(rt)?;

                match opcode {
                    0x0 => Instruction::Load { rt, addr: imm },
                    0x1 => Instruction::Store { rt, addr: imm },
                    0x2 => Instruction::AddImmediate { rt, imm },
                    0x3 => Instruction::AndImmediate { rt, imm },
                    0x4 => Instruction::OrImmediate { rt, imm },
                    0x5 => Instruction::LoadUperImmediate { rt, imm },
                    0x6 => Instruction::CmpImmediate { rt, imm },
                    _ => return Err(format!("invalid I-type instruction: OP=0x{:X}", opcode)),
                }
            }

            Word::JType { opcode, offset } => {
                use Jump::*;
                match opcode {
                    0x7 => Instruction::Jump {
                        jump_type: Call,
                        offset,
                    },
                    0x8 => Instruction::Jump {
                        jump_type: Unconditional,
                        offset,
                    },
                    0x9 => Instruction::Jump {
                        jump_type: Zero,
                        offset,
                    },
                    0xA => Instruction::Jump {
                        jump_type: NotZero,
                        offset,
                    },
                    0xB => Instruction::Jump {
                        jump_type: GreaterThan,
                        offset,
                    },
                    _ => return Err(format!("invalid J-type instruction: OP=0x{:X}", opcode)),
                }
            }

            Word::EType { subcode, rs, rt } => match subcode {
                0x0 => Instruction::Nop,
                0xE => Instruction::Sysall,
                0xF => Instruction::Halt,
                0x1 => {
                    validate_reg(rs)?;
                    validate_spec(rt)?;
                    Instruction::RMovs { rt: rs, spec: rt }
                }
                0x2 => {
                    validate_reg(rt)?;
                    validate_spec(rs)?;
                    Instruction::WMovs { rt, spec: rs }
                }
                _ => return Err(format!("invalid E-type instruction: SUB=0x{:X}", subcode)),
            },
        };

        Ok(instrruction)
    }
}
