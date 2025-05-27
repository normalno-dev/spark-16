use super::{instructions::register::Register, memory::Memory, Flags, CPU};

impl CPU {
    pub fn get_registers(&self) -> &[u16] {
        &self.registers
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn get_flags(&self) -> &Flags {
        &self.flags
    }

    pub fn get_memory(&self) -> &Memory {
        &self.memory
    }

    pub fn dump_registers(&self) -> String {
        let mut output = String::new();
        let registers = [
            Register::R0,
            Register::R1,
            Register::R2,
            Register::R3,
            Register::R4,
            Register::R5,
            Register::R6,
            Register::R7,
            Register::PC,
            Register::SP,
            Register::FLAGS,
        ];

        output.push_str(&format!("REG     | HEX    | BIN                | DEC\n"));
        output.push_str(&format!("--------|--------|--------------------|----\n"));
        for reg in registers.iter() {
            match reg {
                Register::FLAGS => {
                    let val = self.get_register(*reg);
                    output.push_str(&format!("{}\t| 0x{:04X} | 0b{:016b} | {}\n", reg, val, val, self.flags));                    
                },
                _ => {
                    let val = self.get_register(*reg);
                    output.push_str(&format!("{}\t| 0x{:04X} | 0b{:016b} | {:5}\n", reg, val, val, val));
                }
            }
        }

        output
    }

    pub fn dump_memory_hex(&self, start: u16, length: u16) -> String {
        let mut output = String::new();
        let data = self.memory.get_range(start, length);

        for (i, byte) in data.iter().enumerate() {
            if i % 16 == 0 {
                output.push_str(&format!("0x{:04X}: ", start + i as u16));
            }

            output.push_str(&format!("0x{:02X} ", byte));
            if i % 16 == 15 {
                output.push('\n');
            }
        }

        output
    }
}
