use crate::isa::Op;

pub fn disassemble(code: &[u8]) -> Result<String, String> {
    let mut out = String::new();
    let mut ip = 0;
    while ip < code.len() {
        let (op, next_ip) = Op::decode(code, ip)?;
        ip = next_ip;
        let line = match op {
            Op::Push(value) => format!("PUSH {}\n", value),
            Op::Pop => "POP\n".to_string(),
            Op::Dup => "DUP\n".to_string(),
            Op::Swap => "SWAP\n".to_string(),
            Op::Add => "ADD\n".to_string(),
            Op::Sub => "SUB\n".to_string(),
            Op::Mul => "MUL\n".to_string(),
            Op::Div => "DIV\n".to_string(),
            Op::Mod => "MOD\n".to_string(),
            Op::Neg => "NEG\n".to_string(),
            Op::Load(slot) => format!("LOAD {}\n", slot),
            Op::Store(slot) => format!("STORE {}\n", slot),
            Op::Print => "PRINT\n".to_string(),
            Op::Halt => "HALT\n".to_string(),
        };
        out.push_str(&line);
    }
    Ok(out)
}
