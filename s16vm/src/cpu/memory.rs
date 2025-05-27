use super::error::CpuError;

#[derive(Debug)]
pub enum MemoryError {
    OutOfBounds(u16),
}

type Result<T> = std::result::Result<T, MemoryError>;

pub struct Memory {
    data: [u8; 0x10000], // 64KB, [0x0000,0xFFFF]
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
            return Err(MemoryError::OutOfBounds(address));
        }

        // Little-endian: LSB at lower address
        let low = self.data[addr] as u16;
        let high = self.data[addr + 1] as u16;
        let word = (high << 8) | low;
        return Ok(word);
    }

    pub fn write_word(&mut self, address: u16, value: u16) -> Result<()> {
        let addr = address as usize;

        if addr >= 0x10000 - 1 {
            return Err(MemoryError::OutOfBounds(address));
        }

        // Little-endian: LSB at lower address
        let low = (value & 0x00FF) as u8;
        let high = (value >> 8) as u8;
        self.data[addr] = low;
        self.data[addr + 1] = high;

        Ok(())
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) -> Result<()> {
        let addr = address as usize;

        if addr >= 0x10000 {
            return Err(MemoryError::OutOfBounds(address));
        }

        self.data[addr] = byte;
        Ok(())
    }

    pub fn get_range(&self, start: u16, length: u16) -> Vec<u8> {
        let start_idx = start as usize;
        let end = start as u32 + length as u32;

        if end > 0xFFFF {
            Vec::new()
        } else {
            let end_idx = (start + length) as usize;
            self.data[start_idx..end_idx].to_vec()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_write_word() {
        // Test writing LSB and MSB bytes
        {
            let mut mem = Memory::default();
            mem.write_word(0xA, 0xABCD).unwrap();

            assert_eq!(mem.data[0xA], 0xCD); // LSB at lower address
            assert_eq!(mem.data[0xB], 0xAB); // MSB
        }

        // Test writing zero at zero
        {
            let mut mem = Memory::default();
            mem.write_word(0x0, 0x0).unwrap();

            assert_eq!(mem.data[0x0], 0x0);
            assert_eq!(mem.data[0x1], 0x0);
        }

        // Test writing at maximal address
        {
            let mut mem = Memory::default();
            mem.write_word(0xFFFE, 0xABCD).unwrap();

            assert_eq!(mem.data[0xFFFE], 0xCD); // LSB at lower address
            assert_eq!(mem.data[0xFFFF], 0xAB); // MSB
        }

        // Test writing one byte value
        {
            let mut memory = Memory::default();

            // Test writing when MSB is zero
            memory.write_word(0x50, 0x42).unwrap();
            assert_eq!(memory.data[0x50], 0x42);
            assert_eq!(memory.data[0x51], 0x0);

            // Test writing when LSB is zero
            memory.write_word(0x50, 0x4200).unwrap();
            assert_eq!(memory.data[0x50], 0x0);
            assert_eq!(memory.data[0x51], 0x42);
        }

        // Test writing out of bounds
        {
            let mut mem = Memory::default();
            let result = mem.write_word(0xFFFF, 0xABCD);
            assert!(result.is_err());

            if let Err(MemoryError::OutOfBounds(addr)) = result {
                assert_eq!(addr, 0xFFFF);
            } else {
                panic!("Expected OutOfBounds error");
            }
        }
    }

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
