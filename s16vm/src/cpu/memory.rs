use super::error::CpuError;

#[derive(Debug)]
pub enum MemoryError {
    OutOfBounds(u16)
}

type Result<T> = std::result::Result<T, MemoryError>;

pub struct Memory {
    data: [u8; 0x10000], // 64KB
}

impl Default for Memory {
    fn default() -> Self {
        Self { data: [0; 0x10000] }
    }
}

impl Memory {
    pub fn read_word(&self, address: u16) -> Result<u16> {
        let addr = address as usize;

        if addr >= 0x10000 - 1 {
            return Err(MemoryError::OutOfBounds(address))
        }

        // Little-endian: LSB at lower address
        let low = self.data[addr] as u16;
        let high = self.data[addr+1] as u16;
        let word = (high << 8) | low;
        return Ok(word)
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) -> Result<()> {
        let addr = address as usize;

        if addr >= 0x10000 {
            return Err(MemoryError::OutOfBounds(address))
        }

        self.data[addr] = byte;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_word() {
        // Test reading LSB and MSB bytes
        {
            let mut mem = Memory::default();
            mem.data[0x10] = 0xCD; // LSB at lower address
            mem.data[0x11] = 0xAB; // MSB

            let result = mem.read_word(0x10).unwrap();
            assert_eq!(result, 0xABCD)
        }

        // Test reading zero at zero
        {
            let mem = Memory::default();
            let result = mem.read_word(0x0).unwrap();
            assert_eq!(result, 0x0000);
        }

        // Test reading at maximal address
        {
            let mut mem = Memory::default();
            mem.data[0xFFFE] = 0xCD;
            mem.data[0xFFFF] = 0xAB;

            let result = mem.read_word(0xFFFE).unwrap();
            assert_eq!(result, 0xABCD)
        }

        // Test reading one byte value
        {
            let mut memory = Memory::default();

            // Test reading when MSB is zero
            memory.data[0x50] = 0x42; // LSB
            memory.data[0x51] = 0x00; // MSB = 0
            assert_eq!(memory.read_word(0x50).unwrap(), 0x0042);
            
            // Test reading when LSB is zero
            memory.data[0x60] = 0x00; // LSB = 0
            memory.data[0x61] = 0x42; // MSB
            assert_eq!(memory.read_word(0x60).unwrap(), 0x4200);
        }

        // Test reading out of bounds
        {
            let mem = Memory::default();
            let result = mem.read_word(0xFFFF);
            assert!(result.is_err());

            if let Err(MemoryError::OutOfBounds(addr)) = result {
                assert_eq!(addr, 0xFFFF);
            } else {
                panic!("Expected OutOfBounds error");
            }
        }
    }
}