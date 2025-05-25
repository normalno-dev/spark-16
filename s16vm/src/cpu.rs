mod instructions;

pub enum StatusRegister {
    Z = 0x0, // Zero flag
    N = 0x1, // Negative flag
    C = 0x2, // Carry flag
    V = 0x3, // Overflow flag
}

pub struct S16VM {
    pub registers: [u16; 7],  // 7 general-purpose registers R0-R7
    pub pc: u16,              // Program Counter
    pub sp: u16,              // Stack Pointer
    pub sr: StatusRegister,   // Status register (Z, C, N, V)

    pub memory: [u8; 65536], // 64KB of memory
}

fn foo() {
    let sr = StatusRegister::C;
}
