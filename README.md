# MiniVM

## Overview

MiniVM is a stack-based bytecode virtual machine implemented in Rust as part of a systems programming assignment.

The project consists of three components:

* **Assembler** (`minivm asm`) – converts assembly (`.tasm`) into bytecode (`.tbc`)
* **Virtual Machine** (`minivm run`) – executes bytecode
* **Disassembler** (`minivm dis`) – converts bytecode back into assembly



---

# Building

Build the project:

```bash
cargo build
```

Build an optimized release binary:

```bash
cargo build --release
```

Run all tests:

```bash
cargo test
```

---

# Running

## Assemble

```bash
cargo run -- asm input.tasm -o output.tbc
```

## Execute

```bash
cargo run -- run output.tbc
```

## Execute with Trace Mode

```bash
cargo run -- run --trace output.tbc
```

## Disassemble

```bash
cargo run -- dis output.tbc
```

---

# Project Pipeline

```
program.tasm
      │
      ▼
  Assembler
      │
      ▼
 program.tbc
      │
      ▼
 Virtual Machine
      │
      ▼
   Program Output
```

or

```
program.tbc
      │
      ▼
 Disassembler
      │
      ▼
program.tasm
```

---

# Acceptance Tests

The following acceptance requirements from the specification have been completed.

| Requirement                                
| ------------------------------------------ 
| Assemble `.tasm` to `.tbc`                
| Execute bytecode                           
| Trace mode (`--trace`)                     
| Disassembler                               
| Byte-identical `asm → dis → asm`           
| Bytecode validation (magic/version/length) 
| Runtime trap handling                      
| Integration tests                          

Programs tested include:

* Arithmetic example
* Horner polynomial evaluation
* Celsius to Fahrenheit conversion
* Stack manipulation example
* Digit extraction
* Trap test programs
* Round-trip assembler/disassembler verification

The project also includes integration tests covering:

* Assembler
* Bytecode reader/writer
* Virtual machine
* Runtime traps
* Disassembler
* Round-trip verification

Run all tests:

```bash
cargo test
```

---

# Required Translation Table (Infix → Stack Code)

Expression:

```
(7 + 3) * (9 - 4) / 5
```

| Infix Operation | Stack Machine Code |
| --------------- | ------------------ |
| Push 7          | `PUSH 7`           |
| Push 3          | `PUSH 3`           |
| Compute 7 + 3   | `ADD`              |
| Push 9          | `PUSH 9`           |
| Push 4          | `PUSH 4`           |
| Compute 9 - 4   | `SUB`              |
| Multiply        | `MUL`              |
| Push 5          | `PUSH 5`           |
| Divide          | `DIV`              |
| Print result    | `PRINT`            |
| Stop execution  | `HALT`             |

Resulting assembly:

```asm
PUSH 7
PUSH 3
ADD
PUSH 9
PUSH 4
SUB
MUL
PUSH 5
DIV
PRINT
HALT
```

Output:

```
10
```

---

# Example Trace

```
ip=0 PUSH 7 stack=[]
ip=9 PUSH 3 stack=[7]
ip=18 ADD stack=[7, 3]
ip=19 PRINT stack=[10]
10
ip=20 HALT stack=[]
```

---

# Runtime Traps

The VM reports runtime errors together with the instruction pointer.

Implemented traps:

* Stack underflow
* Stack overflow
* Divide by zero
* Modulo by zero
* Unknown opcode
* Truncated instruction
* Missing `HALT`

Example:

```
trap at ip=0x0012: divide by zero
```

---

