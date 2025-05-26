use std::fmt::UpperHex;

const OPCODE_SHIFT: u8 = 12;
const RD_SHIFT: u8 = 9;
const RS_SHIFT: u8 = 6;
const RT_SHIFT: u8 = 3;

const OPCODE_MASK: u16    = 0b1111000000000000;
const RD_MASK: u16        = 0b0000111000000000;
const RS_MASK: u16        = 0b0000000111000000;
const RT_MASK: u16        = 0b0000000000111000;
const FUNCT_MASK: u16     = 0b0000000000000111;
const IMMEDIATE_MASK: u16 = 0b0000000111111111;
const OFFSET_MASK: u16    = 0b0000111111111111;

// A single word can code 4 different insturction types:
// **R-Type (Register-Register Operations)**
// ```
// 15 14 13 12 | 11 10 09 | 08 07 06 | 05 04 03 | 02 01 00
// OPCODE      | RD       | RS       | RT       | FUNCT
// ```

// **I-Type (Immediate Operations)**
// ```
// 15 14 13 12 | 11 10 09 | 08 07 06 05 04 03 02 01 00
// OPCODE      | RT       | IMMEDIATE (9-bit signed)
// ```

// **J-Type (Jump Operations)**
// ```
// 15 14 13 12 | 11 10 09 08 07 06 05 04 03 02 01 00
// OPCODE      | OFFSET (12-bit signed)
// ```

// **E-Type (Extended Operations)**
// ```
// 15 14 13 12 | 11 10 09 | 08 07 06 | 05 04 03 | 02 01 00
// 0xF         | SUBCODE  | RS       | RT       | 0x0
// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    RType { opcode: u8, rd: u8, rs: u8, rt: u8, funct: u8 },
    IType { opcode: u8, rt: u8, imm: u8 },
    JType { opcode: u8, offset: i16 },
    EType { subcode: u8, rs: u8, rt: u8 },
}

impl UpperHex for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.to_bits();
        write!(f, "{:X}", bits)
    }
}

impl Word {
    pub fn new(bits: u16) -> Self {
        let opcode = ((bits & OPCODE_MASK) >> OPCODE_SHIFT) as u8;

        match opcode {
            0x0..=0x1 => Self::RType {
                opcode,
                rd: ((bits & RD_MASK) >> RD_SHIFT) as u8,
                rs: ((bits & RS_MASK) >> RS_SHIFT) as u8,
                rt: ((bits & RT_MASK) >> RT_SHIFT) as u8,
                funct: (bits & FUNCT_MASK) as u8,
            },
            0x2..=0x8 => Self::IType {
                opcode,
                rt: ((bits & RD_MASK) >> RD_SHIFT) as u8,
                imm: (bits & IMMEDIATE_MASK) as u8,
            },
            0x9..=0xD => Self::JType {
                opcode,
                offset: (bits & OFFSET_MASK) as i16,
            },
            0xF => Self::EType {
                subcode: ((bits & RD_MASK) >> RD_SHIFT) as u8,
                rs: ((bits & RS_MASK) >> RS_SHIFT) as u8,
                rt: ((bits & RT_MASK) >> RT_SHIFT) as u8,
            },
            _ => Self::RType { opcode: 0, rd: 0, rs: 0, rt: 0, funct: 0 }
        }
    }

    pub fn opcode(self) -> u8 {
        match self {
            Self::RType { opcode, .. }
            | Self::IType { opcode, .. }
            | Self::JType { opcode, .. } => opcode,
            Self::EType { .. } => 0xF,
        }
    }

    pub fn subcode(self) -> Option<u8> {
        match self {
            Self::EType { subcode, .. } => Some(subcode),
            _ => None,
        }
    }

    pub fn immediate(self) -> Option<u8> {
        match self {
            Self::IType { imm: immediate, .. } => Some(immediate),
            _ => None,
        }
    }

    pub fn offset(self) -> Option<i16> {
        match self {
            Self::JType { offset, .. } => Some(offset),
            _ => None,
        }
    }

    pub fn rd(self) -> Option<u8> {
        match self {
            Self::RType { rd, .. } => Some(rd),
            _ => None,
        }
    }

    pub fn rs(self) -> Option<u8> {
        match self {
            Self::RType { rs, .. }
            | Self::EType { rs, .. } => Some(rs),
            _ => None,
        }
    }

    pub fn rt(self) -> Option<u8> {
        match self {
            Self::RType { rt,..}
            | Self::IType { rt, ..}
            | Self::EType { rt, ..} => Some(rt),
            _ => None
        }
    }

    pub fn funct(self) -> Option<u8> {
        match self {
            Self::RType { funct, .. } => Some(funct),
            _ => None,
        }
    }
}

impl Word {
    pub fn to_bits(self) -> u16 {
        match self {
            Self::RType { opcode, rd, rs, rt, funct } => {
                (opcode as u16) << OPCODE_SHIFT |
                (rd as u16) << RD_SHIFT |
                (rs as u16) << RS_SHIFT |
                (rt as u16) << RT_SHIFT |
                (funct as u16)
            },

            Self::IType { opcode, rt, imm: immediate } => {
                (opcode as u16) << OPCODE_SHIFT |
                (rt as u16) << RD_SHIFT |
                (immediate as u16)
            },

            Self::JType { opcode, offset } => {
                (opcode as u16) << OPCODE_SHIFT |
                (offset as u16)
            },

            Self::EType { subcode, rs, rt } => {
                (0xF << OPCODE_SHIFT) |
                (subcode as u16) << RD_SHIFT |
                (rs as u16) << RS_SHIFT |
                (rt as u16) << RT_SHIFT
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upperhex_fmt() {
        let word = Word::RType {
            opcode: 0x1,
            rd: 0x2,
            rs: 0x3,
            rt: 0x4,
            funct: 0x5,
        };
        let hex = format!("{:X}", word);
        assert_eq!(u16::from_str_radix(&hex, 16).unwrap(), word.to_bits());
    }

    #[test]
    fn test_new() {
        {
            let w = Word::new(0b0001_101_110_111_010);
            assert_eq!(w, Word::RType { opcode: 0b0001, rd: 0b0101, rs: 0b0110, rt: 0b0111, funct: 0b0010 });
        }

        {
            let w = Word::new(0b0110_101_011111111);
            assert_eq!(w, Word::IType { opcode: 0b0110, rt: 0b0101, imm: 0b011111111 });
        }

        {
            let w = Word::new(0b1010_1100_1101_1110);
            assert_eq!(w, Word::JType { opcode: 0xA, offset: 0xCDE });
        }

        {
            let w = Word::new(0b1111_110_010_011_000);
            assert_eq!(w, Word::EType { subcode: 0b110, rs: 0b010, rt: 0b011 });
        }
    }
    
    #[test]
    fn test_to_bits() {
        {
            let original_bits = 0x1234_u16;
            let word = Word::new(original_bits);
            assert_eq!(word.to_bits(), original_bits);
        }

        {
            let word = Word::RType { opcode: 0x1, rd: 0x5, rs: 0x6, rt: 0x7, funct: 0x2 };
            let bits = word.to_bits();
            assert_eq!(bits, 0b0001_101_110_111_010);
        }

        {
            let word = Word::JType { opcode: 0xA, offset: 0xCDE };
            let bits = word.to_bits();
            assert_eq!(bits, 0b1010_1100_1101_1110);
        }

        {
            let word = Word::IType { opcode: 0x6, rt: 0x5, imm: 0xFF };
            let bits = word.to_bits();
            assert_eq!(bits, 0b0110_101_011111111);
        }

        {
            let word = Word::EType { subcode: 0x6, rs: 0x2, rt: 0x3 };
            let bits = word.to_bits();
            assert_eq!(bits, 0b1111_110_010_011_000);
        }
    }
}
