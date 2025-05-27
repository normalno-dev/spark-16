mod debug;
mod implementations;
mod memory;
mod types;

pub mod error;
pub mod instructions;

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

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Z:{} C:{} N:{} V:{}]", self.zero as u8, self.carry as u8, self.negative as u8, self.overflow as u8)
    }
}

impl Flags {
    fn as_u16(&self) -> u16 {
        (self.zero as u16) << 0
            | (self.carry as u16) << 1
            | (self.negative as u16) << 2
            | (self.overflow as u16) << 3
    }

    fn from_u16(&mut self, value: u16) {
        self.zero = (value & 0x01) != 0;
        self.carry = (value & 0x02) != 0;
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
    pub fn new() -> Self {
        Self {
            registers: [0; 8],
            pc: 0x0,
            sp: 0xFFFF,
            flags: Flags::default(),
            memory: Memory::default(),
            halted: false,
            program_start: 0x0,
            program_end: 0x0,
        }
    }
    pub fn step(&mut self) -> Result<bool> {
        if self.halted {
            return Ok(false);
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
        println!("{:04X}: {}", self.pc-2, instruction);
        self.execute(instruction)?;

        Ok(true)
    }

    // Load a program into memory with starting address start_addr.
    // Automatically sets program boundaries and pc.
    pub fn load_program(&mut self, program: Vec<u8>, start_addr: u16) -> Result<()> {
        let program_size = program.len() as u16;

        self.program_start = start_addr;
        self.program_end = start_addr + program_size;
        self.halted = false;
        self.pc = start_addr;
        self.sp = 0xFFFE;

        for (i, &byte) in program.iter().enumerate() {
            let addr = start_addr + i as u16;
            self.memory.write_byte(addr, byte)?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.step()? {}

        Ok(())
    }

    // Control program boundries, it's the simplest way to not fuck up.
    // Later, it should be upgraded to hybrid system based on memory segments and CPU security polices.
    fn secure_boundaries(&self) -> Result<()> {
        // Check if we can read full instruction and not became out of program boundaries.
        let instruction_end = self.pc.saturating_add(2);
        if self.pc < self.program_start || instruction_end > self.program_end {
            Err(CpuError::ProgramBoundsViolation {
                pc: self.pc,
                iend: instruction_end,
                low: self.program_start,
                high: self.program_end,
            })
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

    fn update_flags_arithmetic(
        &mut self,
        operand_a: u16,
        operand_b: u16,
        result: u16,
        carry: bool,
        is_substraction: bool,
    ) {
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

    fn check_overflow(
        &self,
        operand_a: u16,
        operand_b: u16,
        result: u16,
        is_substraction: bool,
    ) -> bool {
        let a_sign = (operand_a & 0x8000) != 0;
        let b_sign = (operand_b & 0x8000) != 0;
        let result_sign = (result & 0x8000) != 0;

        if is_substraction {
            // overflow happens when:
            // 1. positive - negative = negative
            // 2. negative - positive = positive
            (!a_sign && b_sign && result_sign) || (a_sign && !b_sign && !result_sign)
        } else {
            // overflow happens when:
            // 1. negative + negative = positive
            // 2. positive + positive = negative
            (!a_sign && !b_sign && result_sign) || (a_sign && b_sign && !result_sign)
        }
    }
}
