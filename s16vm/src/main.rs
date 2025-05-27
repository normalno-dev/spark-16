mod cpu;

use cpu::{error::CpuError, instructions::word::Word, CPU};

fn main() -> Result<(), CpuError> {
    let prog = program();
    let mut cpu = CPU::new();
    
    cpu.load_program(prog, 0x10)?;

    let registers = cpu.dump_registers();
    let memory= cpu.dump_memory_hex(0x10, 0x10);
    println!("{}", registers);
    println!("{}", memory);

    cpu.run()?;

    let registers = cpu.dump_registers();
    let memory= cpu.dump_memory_hex(0x10, 0x10);
    println!("{}", registers);
    println!("{}", memory);

    Ok(())
}

fn program() -> Vec<u8> {
    let prog = vec![
        /*ADDI r1, 5     */ Word::IType { opcode: 0x4, rt: 0x1, imm: 0x05 }, 
        /*ADDI r2, 3     */ Word::IType { opcode: 0x4, rt: 0x2, imm: 0x03 },
        /*ADD r3, r1, r2 */ Word::RType { opcode: 0x0, rd: 0x3, rs: 0x1, rt: 0x2, funct: 0x0 },
        /*STORE r3, 0x1F */ Word::IType { opcode: 0x3, rt: 0x3, imm: 0x1F },
        /*LOAD r3, 0x1F  */ Word::IType { opcode: 0x2, rt: 0x5, imm: 0x1F },
        /*HALT           */ Word::EType { subcode: 0xF, rs: 0x0, rt: 0x0 }
    ];

    let mut bytes = Vec::<u8>::new();
    for w in prog.iter() {
        let bits = w.to_bits();
        let low = ((bits << 8) >> 8) as u8;
        let high = (bits >> 8) as u8;
        bytes.push(low);
        bytes.push(high);
    }

    bytes
}