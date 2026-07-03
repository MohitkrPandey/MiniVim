#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Push(i64),
    Pop,
    Dup,
    Swap,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Load(u8),
    Store(u8),
    Print,
    Halt,
}

impl Op {
    pub fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Op::Push(value) => {
                output.push(0x01);
                output.extend_from_slice(&value.to_le_bytes());
            }
            Op::Pop => output.push(0x02),
            Op::Dup => output.push(0x03),
            Op::Swap => output.push(0x04),
            Op::Add => output.push(0x10),
            Op::Sub => output.push(0x11),
            Op::Mul => output.push(0x12),
            Op::Div => output.push(0x13),
            Op::Mod => output.push(0x14),
            Op::Neg => output.push(0x15),
            Op::Load(index) => {
                output.push(0x40);
                output.push(*index);
            }
            Op::Store(index) => {
                output.push(0x41);
                output.push(*index);
            }
            Op::Print => output.push(0x60),
            Op::Halt => output.push(0xFF),
        }
    }

    pub fn decode(bytes: &[u8], ip: usize) -> Result<(Op, usize), String> {
        let opcode = *bytes.get(ip).ok_or_else(|| "unknown opcode".to_string())?;
        let next = ip + 1;
        match opcode {
            0x01 => {
                let end = next + 8;
                if bytes.len() < end {
                    return Err("truncated PUSH operand".into());
                }
                let mut raw = [0u8; 8];
                raw.copy_from_slice(&bytes[next..end]);
                let value = i64::from_le_bytes(raw);
                Ok((Op::Push(value), end))
            }
            0x02 => Ok((Op::Pop, next)),
            0x03 => Ok((Op::Dup, next)),
            0x04 => Ok((Op::Swap, next)),
            0x10 => Ok((Op::Add, next)),
            0x11 => Ok((Op::Sub, next)),
            0x12 => Ok((Op::Mul, next)),
            0x13 => Ok((Op::Div, next)),
            0x14 => Ok((Op::Mod, next)),
            0x15 => Ok((Op::Neg, next)),
            0x40 => {
                let index = *bytes.get(next).ok_or_else(|| "truncated LOAD operand".to_string())?;
                Ok((Op::Load(index), next + 1))
            }
            0x41 => {
                let index = *bytes.get(next).ok_or_else(|| "truncated STORE operand".to_string())?;
                Ok((Op::Store(index), next + 1))
            }
            0x60 => Ok((Op::Print, next)),
            0xFF => Ok((Op::Halt, next)),
            _ => Err("unknown opcode".into()),
        }
    }
}
