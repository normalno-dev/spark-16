# 16-bit RISC CPU Architecture Documentation

## Overview

This document describes a 16-bit RISC (Reduced Instruction Set Computer) processor designed for educational purposes. The architecture follows RISC principles with fixed-size 16-bit instructions and a simple, orthogonal instruction set inspired by MIPS.

## Register Architecture

### General Purpose Registers
- **8 registers**: R0 - R7 (3-bit addressing)
- **R0**: Always contains zero (hardware-enforced)
- **R1-R7**: General purpose registers, 16-bit each

### Special Registers
- **PC (Program Counter)**: 16-bit instruction pointer
  - Automatically incremented after each instruction
  - Modified by jump and call instructions
- **SP (Stack Pointer)**: 16-bit stack pointer
  - Points to the top of the stack
  - Decremented on PUSH, incremented on POP
  - Stack grows downward in memory
- **FLAGS**: 16-bit status register
  - Bit 0: **Z (Zero)** - Set when result equals zero
  - Bit 1: **C (Carry)** - Set when carry/borrow occurs
  - Bit 2: **N (Negative)** - Set when result is negative (bit 15 = 1)
  - Bit 3: **V (Overflow)** - Set when signed arithmetic overflow occurs
  - Bits 4-15: Reserved (always 0)

### Flag Setting Rules

**Instructions that set flags:**
- **Arithmetic**: ADD, SUB, ADDI → Z, N, C, V
- **Logical**: AND, OR, XOR, NOT, ANDI, ORI → Z, N (C=0, V=0)
- **Shifts**: SLL, SHR → Z, N, C (last shifted bit), V=0
- **Compare**: CMP, CMPI → Z, N, C, V (performs subtraction without storing result)

**Instructions that do NOT affect flags:**
- Memory operations: LOAD, STORE, LOADI, STOREI
- Control flow: JMP, JZ, JNZ, JGT, CALL, RET
- Stack operations: PUSH, POP
- Special register access: MOVS
- System: LUI, NOP, SYSCALL, HALT

## Instruction Formats

### R-Type (Register-Register Operations)
```
15 14 13 12 | 11 10 09 | 08 07 06 | 05 04 03 | 02 01 00
OPCODE      | RD       | RS       | RT       | FUNCT
```

### I-Type (Immediate Operations)
```
15 14 13 12 | 11 10 09 | 08 | 07 06 05 04 03 02 01 00
OPCODE      | RT       | 0  | IMMEDIATE
```
- Immediate range: -128 to +127

### J-Type (Jump Operations)
```
15 14 13 12 | 11 10 09 08 07 06 05 04 03 02 01 00
OPCODE      | OFFSET (12-bit signed)
```
- Jump range: -2048 to +2047 (relative to PC)

### E-Type (Extended Instructions)
```
15 14 13 12 | 11 10 09 08 | 07 06  05 | 04 03 02 | 01 00
0xF         | SUBCODE     | RS        | RT       | 0x0
```

## Instruction Set

### R-Type Instructions

| Mnemonic       | Opcode | Funct | Description                              |
| -------------- | ------ | ----- | ---------------------------------------- |
| ADD Rd, Rs, Rt | 0x0    | 0x0   | Rd = Rs + Rt                             |
| SUB Rd, Rs, Rt | 0x0    | 0x1   | Rd = Rs - Rt                             |
| AND Rd, Rs, Rt | 0x0    | 0x2   | Rd = Rs & Rt (bitwise AND)               |
| OR Rd, Rs, Rt  | 0x0    | 0x3   | Rd = Rs \| Rt (bitwise OR)               |
| XOR Rd, Rs, Rt | 0x0    | 0x4   | Rd = Rs ^ Rt (bitwise XOR)               |
| NOT Rd, Rs     | 0x0    | 0x5   | Rd = ~Rs (bitwise NOT)                   |
| SLL Rd, Rs, Rt | 0x0    | 0x6   | Rd = Rs << Rt (shift left logical)       |
| SHR Rd, Rs, Rt | 0x0    | 0x7   | Rd = Rs >> Rt (shift right logical)      |
| LOADI Rd, Rs   | 0x1    | 0x0   | Rd = Memory[Rs] (load indirect)          |
| STOREI Rd, Rs  | 0x1    | 0x1   | Memory[Rs] = Rd (store indirect)         |
| CMP Rs, Rt     | 0x1    | 0x2   | Compare Rs and Rt, set flags             |
| RET            | 0x1    | 0x3   | Return from function (PC = Memory[SP++]) |
| PUSH Rs        | 0x1    | 0x4   | Memory[--SP] = Rs                        |
| POP Rd         | 0x1    | 0x5   | Rd = Memory[SP++]                        |

### I-Type Instructions

| Mnemonic       | Opcode | Description                          |
| -------------- | ------ | ------------------------------------ |
| LOAD Rt, addr  | 0x2    | Rt = Memory[addr]                    |
| STORE Rt, addr | 0x3    | Memory[addr] = Rt                    |
| ADDI Rt, imm   | 0x4    | Rt = Rt + imm                        |
| ANDI Rt, imm   | 0x5    | Rt = Rt & imm                        |
| ORI Rt, imm    | 0x6    | Rt = Rt \| imm                       |
| LUI Rt, imm    | 0x7    | Rt = imm << 8 (load upper immediate) |
| CMPI Rt, imm   | 0x8    | Compare Rt and imm, set flags        |

### J-Type Instructions

| Mnemonic    | Opcode | Description                         |
| ----------- | ------ | ----------------------------------- |
| CALL offset | 0x9    | Memory[--SP] = PC; PC = PC + offset |
| JMP offset  | 0xA    | PC = PC + offset                    |
| JZ offset   | 0xB    | if (Z flag) PC = PC + offset        |
| JNZ offset  | 0xC    | if (!Z flag) PC = PC + offset       |
| JGT offset  | 0xD    | if (!Z && N==V) PC = PC + offset    |

### E-Type Instructions

| Mnemonic      | Opcode | Subcode | Description                                 |
| ------------- | ------ | ------- | ------------------------------------------- |
| NOP           | 0xF    | 0x0     | No operation                                |
| MOVS Rt, SPEC | 0xF    | 0x1     | Rt = Special Register (PC=0, SP=1, FLAGS=2) |
| MOVS SPEC, Rs | 0xF    | 0x2     | Special Register = Rs (PC=0, SP=1, FLAGS=2) |
| SYSCALL       | 0xF    | 0xE     | System call (implementation-defined)        |
| HALT          | 0xF    | 0xF     | Stop processor execution                    |

## Assembly Language Examples

### Example 1: Simple Arithmetic
```assembly
# Calculate (5 + 3) * 2
ADDI R1, 5          # R1 = 5
ADDI R2, 3          # R2 = 3  
ADD  R3, R1, R2     # R3 = R1 + R2 = 8
SLL  R4, R3, R0     # R4 = R3 << 1 = R3 * 2 = 16
ADDI R4, 1          # Alternative: R4 = R3 + R3
```

### Example 2: Array Sum
```assembly
# Sum array of 5 numbers starting at address 0x100
LUI  R1, 1          # R1 = 0x100 (array base address)
ADDI R2, 5          # R2 = 5 (counter)
ADDI R3, 0          # R3 = 0 (sum)

loop:
    LOADI R4, R1    # R4 = Memory[R1]
    ADD   R3, R3, R4 # sum += R4
    ADDI  R1, 1     # R1++ (next element)
    ADDI  R2, -1    # counter--
    CMPI  R2, 0     # compare counter with 0
    JNZ   loop      # if counter != 0, continue loop

# R3 now contains the sum
```

### Example 3: Function Call
```assembly
# Function that doubles a number
main:
    ADDI R1, 42     # Load argument
    CALL double_fn  # Call function
    # R1 now contains 84
    HALT

double_fn:
    SLL  R1, R1, R0 # R1 = R1 << 1 (multiply by 2)
    ADDI R1, 1      # Alternative: ADD R1, R1, R1
    RET             # Return to caller
```

### Example 4: Conditional Logic
```assembly
# if (a > b) result = a; else result = b (max function)
ADDI R1, 10         # a = 10
ADDI R2, 7          # b = 7

CMP  R1, R2         # Compare a and b
JGT  a_greater      # if a > b, jump to a_greater
ADD  R3, R0, R2     # result = b
JMP  end

a_greater:
    ADD R3, R0, R1  # result = a

end:
    # R3 contains max(a, b) = 10
```

### Example 5: Stack Operations
```assembly
# Save and restore registers
PUSH R1             # Save R1 on stack
PUSH R2             # Save R2 on stack
PUSH R3             # Save R3 on stack

# ... do some work that modifies R1, R2, R3 ...

POP  R3             # Restore R3
POP  R2             # Restore R2  
POP  R1             # Restore R1
```

### Example 6: Working with Special Registers
```assembly
# Debug: print current stack pointer
MOVS R1, SP         # R1 = current stack pointer
# ... output R1 for debugging ...

# Set up initial stack
LUI  R1, 8          # R1 = 0x800
MOVS SP, R1         # SP = 0x800 (initialize stack)

# Check flags after operation
ADDI R1, -1         # R1 = -1 (sets N flag)
MOVS R2, FLAGS      # R2 = current flags
ANDI R2, 4          # Check N flag (bit 2)
JNZ  negative       # Jump if N flag is set
```

## Memory Model

- **16-bit address space**: 64KB total memory (0x0000 - 0xFFFF)
- **Word-addressed**: Each address points to a 16-bit word
- **Little-endian**: Least significant byte at lower address
- **Stack**: Typically located at high memory, grows downward

## Programming Notes

1. **R0 is always zero**: Cannot be modified, useful for constants
2. **Relative addressing**: All jumps are PC-relative
3. **Flag persistence**: Flags remain set until next flag-modifying instruction
4. **Stack management**: CALL automatically saves return address
5. **Immediate limitations**: 9-bit immediates require LUI for larger constants

## Implementation Considerations

- **Pipeline-friendly**: Fixed instruction format simplifies decoding
- **Orthogonal design**: Operations work consistently across register types
- **Minimal instruction set**: Reduces complexity while maintaining functionality
- **Extensible**: E-type format allows future instruction additions