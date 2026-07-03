use crate::isa::Op;

pub fn assemble(input: &str, _output: &str) -> Result<(), String> {
    assemble_bytes(input).map(|_| ())
}

fn assemble_bytes(input: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    for line in input.lines() {
        let content = line.split_once(';').map(|(code, _)| code).unwrap_or(line);
        let trimmed = content.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let mnemonic = parts.next().unwrap().to_ascii_uppercase();
        let op = match mnemonic.as_str() {
            "PUSH" => {
                let operand = parts.next().ok_or_else(|| "missing operand for PUSH".to_string())?;
                if parts.next().is_some() {
                    return Err("unexpected token after PUSH operand".into());
                }
                let value = operand
                    .parse::<i64>()
                    .map_err(|_| "invalid integer for PUSH".to_string())?;
                Op::Push(value)
            }
            "POP" => {
                if parts.next().is_some() {
                    return Err("unexpected token after POP".into());
                }
                Op::Pop
            }
            "DUP" => {
                if parts.next().is_some() {
                    return Err("unexpected token after DUP".into());
                }
                Op::Dup
            }
            "SWAP" => {
                if parts.next().is_some() {
                    return Err("unexpected token after SWAP".into());
                }
                Op::Swap
            }
            "ADD" => {
                if parts.next().is_some() {
                    return Err("unexpected token after ADD".into());
                }
                Op::Add
            }
            "SUB" => {
                if parts.next().is_some() {
                    return Err("unexpected token after SUB".into());
                }
                Op::Sub
            }
            "MUL" => {
                if parts.next().is_some() {
                    return Err("unexpected token after MUL".into());
                }
                Op::Mul
            }
            "DIV" => {
                if parts.next().is_some() {
                    return Err("unexpected token after DIV".into());
                }
                Op::Div
            }
            "MOD" => {
                if parts.next().is_some() {
                    return Err("unexpected token after MOD".into());
                }
                Op::Mod
            }
            "NEG" => {
                if parts.next().is_some() {
                    return Err("unexpected token after NEG".into());
                }
                Op::Neg
            }
            "LOAD" => {
                let operand = parts.next().ok_or_else(|| "missing operand for LOAD".to_string())?;
                if parts.next().is_some() {
                    return Err("unexpected token after LOAD operand".into());
                }
                let slot = operand
                    .parse::<u8>()
                    .map_err(|_| "invalid slot for LOAD".to_string())?;
                Op::Load(slot)
            }
            "STORE" => {
                let operand = parts.next().ok_or_else(|| "missing operand for STORE".to_string())?;
                if parts.next().is_some() {
                    return Err("unexpected token after STORE operand".into());
                }
                let slot = operand
                    .parse::<u8>()
                    .map_err(|_| "invalid slot for STORE".to_string())?;
                Op::Store(slot)
            }
            "PRINT" => {
                if parts.next().is_some() {
                    return Err("unexpected token after PRINT".into());
                }
                Op::Print
            }
            "HALT" => {
                if parts.next().is_some() {
                    return Err("unexpected token after HALT".into());
                }
                Op::Halt
            }
            _ => return Err(format!("unknown instruction: {}", mnemonic)),
        };

        op.encode(&mut bytes);
    }

    Ok(bytes)
}
