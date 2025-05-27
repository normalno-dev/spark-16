use super::{error::CpuError, instructions::{register::Register, Instruction, Jump}, types, CPU};

type Result<T> = std::result::Result<T, CpuError>;

enum LogicalOperation {
    And,
    Or,
    Xor,
    Not,
}

enum ShiftOperation {
    Left,
    Right,
}

// Instruction implementations
impl CPU {
    pub fn execute(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Add { rd, rs, rt } => self.op_add(rd, rs, rt),
            Instruction::Sub { rd, rs, rt } => self.op_sub(rd, rs, rt),

            Instruction::And { rd, rs, rt } => self.op_logical(rd, rs, rt, LogicalOperation::And),
            Instruction::Or { rd, rs, rt } => self.op_logical(rd, rs, rt, LogicalOperation::Or),
            Instruction::Xor { rd, rs, rt } => self.op_logical(rd, rs, rt, LogicalOperation::Xor),
            Instruction::Not { rd, rt } => {
                self.op_logical(rd, Register::R0, rt, LogicalOperation::Not)
            }

            Instruction::Sll { rd, rs, rt } => self.op_shift(rd, rs, rt, ShiftOperation::Left),
            Instruction::Shr { rd, rs, rt } => self.op_shift(rd, rs, rt, ShiftOperation::Right),

            Instruction::LoadIndirect { rd, rs } => self.op_load_indirect(rd, rs),
            Instruction::StoreIndirect { rd, rs } => self.op_store_indirect(rd, rs),

            Instruction::Push { rs } => self.op_push(rs),
            Instruction::Pop { rd } => self.op_pop(rd),

            Instruction::Cmp { rs, rt } => self.op_cmp(rs, rt),
            Instruction::Jump { jump_type, offset } => self.op_jump(jump_type, offset),
            Instruction::Return => self.op_ret(),

            Instruction::AddImmediate { rt, imm } => self.op_add_immediate(rt, imm),
            Instruction::AndImmediate { rt, imm } => self.op_and_immediate(rt, imm),
            Instruction::OrImmediate { rt, imm } => self.op_or_immediate(rt, imm),
            Instruction::CmpImmediate { rt, imm } => self.op_cmp_immediate(rt, imm),
            Instruction::LoadUperImmediate { rt, imm } => self.op_load_upper_immediate(rt, imm),

            Instruction::Load { rt, addr } => self.op_load(rt, addr),
            Instruction::Store { rt, addr } => self.op_store(rt, addr),

            Instruction::MoveFromSpecial { rt, spec } => self.op_movs(rt, spec, false),
            Instruction::MoveFromToSpecial { rt, spec } => self.op_movs(rt, spec, true),

            Instruction::Nop => Ok(()),
            Instruction::Halt => self.op_halt(),
            Instruction::Sysall => Err(CpuError::NotImplementedYet),
        }
    }

    fn op_add(&mut self, rd: Register, rs: Register, rt: Register) -> Result<()> {
        let a = self.get_register(rs);
        let b = self.get_register(rt);
        let (result, carry) = a.overflowing_add(b);

        self.set_register(rd, result);
        self.update_flags_arithmetic(a, b, result, carry, false);

        Ok(())
    }

    fn op_sub(&mut self, rd: Register, rs: Register, rt: Register) -> Result<()> {
        let a = self.get_register(rs);
        let b = self.get_register(rt);
        let (result, borrow) = a.overflowing_sub(b);

        self.set_register(rd, result);
        self.update_flags_arithmetic(a, b, result, borrow, true);

        Ok(())
    }

    fn op_logical(
        &mut self,
        rd: Register,
        rs: Register,
        rt: Register,
        op: LogicalOperation,
    ) -> Result<()> {
        let a = self.get_register(rs);
        let b = self.get_register(rt);
        let result = match op {
            LogicalOperation::And => a & b,
            LogicalOperation::Or => a | b,
            LogicalOperation::Xor => a ^ b,
            LogicalOperation::Not => !a,
        };

        self.set_register(rd, result);
        self.update_flags_logical(result);

        Ok(())
    }

    fn op_shift(
        &mut self,
        rd: Register,
        rs: Register,
        rt: Register,
        op: ShiftOperation,
    ) -> Result<()> {
        let value = self.get_register(rs);
        let s_size = self.get_register(rt) & 0xF; // Mask to 4 bits (0-15 range)

        if s_size == 0 {
            self.set_register(rd, value);
            return Ok(());
        }

        let (result, last_shifted_bit) = match op {
            ShiftOperation::Left => {
                (
                    value << s_size,
                    ((value >> (16 - s_size)) & 1 == 1), // when shift to the left, a bigger bit falls out
                )
            }
            ShiftOperation::Right => {
                (
                    value >> s_size,
                    ((value >> (s_size - 1)) & 1 == 1), // when shift to the left, a lower bit falls out
                )
            }
        };

        self.set_register(rd, result);
        self.update_flags_shift(result, last_shifted_bit);

        Ok(())
    }

    fn op_halt(&mut self) -> Result<()> {
        self.halted = true;
        Ok(())
    }

    fn op_load_indirect(&mut self, rd: Register, rs: Register) -> Result<()> {
        let value = self.memory.read_word(rs as u16)?;
        self.set_register(rd, value);
        Ok(())
    }

    fn op_store_indirect(&mut self, rd: Register, rs: Register) -> Result<()> {
        let value = self.get_register(rd);
        self.memory.write_word(rs as u16, value)?;
        Ok(())
    }

    fn op_cmp(&mut self, rs: Register, rt: Register) -> Result<()> {
        let operand_a = self.get_register(rs);
        let operand_b = self.get_register(rt);
        let (result, borrow) = operand_a.overflowing_sub(operand_b);

        self.update_flags_arithmetic(operand_a, operand_b, result, borrow, true);

        Ok(())
    }

    fn op_push(&mut self, rs: Register) -> Result<()> {
        let value = self.get_register(rs);
        if self.sp < 2 {
            return Err(CpuError::StackOverflow);
        }

        self.sp = self.sp.wrapping_sub(2);
        self.memory.write_word(self.sp, value)?;

        Ok(())
    }

    fn op_pop(&mut self, rd: Register) -> Result<()> {
        if self.sp > 0xFFFE {
            return Err(CpuError::StackOverflow);
        }

        let value = self.memory.read_word(self.sp)?;
        self.set_register(rd, value);
        self.sp = self.sp.wrapping_add(2);

        Ok(())
    }

    fn op_ret(&mut self) -> Result<()> {
        self.op_pop(Register::PC)?;

        Ok(())
    }

    // Remember offset is 12 bits!
    fn op_jump(&mut self, jt: Jump, offset: u16) -> Result<()> {
        let signed_offset = types::convert_12bit_to_signed(offset);

        match jt {
            Jump::Call => {
                self.op_push(Register::PC)?;
                self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
            }
            Jump::Unconditional => {
                self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
            }
            Jump::Zero => {
                if self.flags.zero {
                    self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
                }
            }
            Jump::NotZero => {
                if !self.flags.zero {
                    self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
                }
            }
            Jump::GreaterThan => {
                if !self.flags.zero && self.flags.negative == self.flags.overflow {
                    self.set_register(Register::PC, self.pc.wrapping_add_signed(signed_offset));
                }
            }
        }

        Ok(())
    }

    fn op_add_immediate(&mut self, rt: Register, imm: i8) -> Result<()> {
        let operand_a = self.get_register(rt);
        let operand_b = imm as u16;
        let (result, carry) = operand_a.overflowing_add(operand_b);

        self.set_register(rt, result);
        self.update_flags_arithmetic(operand_a, operand_b, result, carry, false);

        Ok(())
    }

    fn op_and_immediate(&mut self, rt: Register, imm: u8) -> Result<()> {
        let value = self.get_register(rt);
        let imm = imm as u16;
        let result = value & imm;

        self.set_register(rt, result);
        self.update_flags_logical(result);

        Ok(())
    }

    fn op_or_immediate(&mut self, rt: Register, imm: u8) -> Result<()> {
        let value = self.get_register(rt);
        let imm = imm as u16;
        let result = value & imm;

        self.set_register(rt, result);
        self.update_flags_logical(result);

        Ok(())
    }

    fn op_cmp_immediate(&mut self, rt: Register, imm: i8) -> Result<()> {
        let operand_a = self.get_register(rt);
        let operand_b = imm as u16;
        let (result, borrow) = operand_a.overflowing_sub(operand_b);

        self.update_flags_arithmetic(operand_a, operand_b, result, borrow, true);

        Ok(())
    }

    // imm is used as 8 bits value
    fn op_load_upper_immediate(&mut self, rt: Register, imm: u8) -> Result<()> {
        let value = (imm as u16) << 8;
        self.set_register(rt, value);

        Ok(())
    }

    fn op_load(&mut self, rt: Register, addr: u8) -> Result<()> {
        let value = self.memory.read_word(addr as u16)?;
        self.set_register(rt, value);

        Ok(())
    }

    fn op_store(&mut self, rt: Register, addr: u8) -> Result<()> {
        let value = self.get_register(rt);
        self.memory.write_word(addr as u16, value)?;

        Ok(())
    }

    fn op_movs(&mut self, rt: Register, spec: Register, to_special: bool) -> Result<()> {
        let (source, target) = match to_special {
            true => (rt, spec),
            false => (spec, rt),
        };

        let value = self.get_register(source);
        self.set_register(target, value);

        Ok(())
    }
}
