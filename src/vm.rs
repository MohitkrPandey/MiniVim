use crate::isa::Op;
use crate::bytecode;

fn format_op(op: Op) -> String {
    match op {
        Op::Push(value) => format!("PUSH {}", value),
        Op::Pop => "POP".to_string(),
        Op::Dup => "DUP".to_string(),
        Op::Swap => "SWAP".to_string(),
        Op::Add => "ADD".to_string(),
        Op::Sub => "SUB".to_string(),
        Op::Mul => "MUL".to_string(),
        Op::Div => "DIV".to_string(),
        Op::Mod => "MOD".to_string(),
        Op::Neg => "NEG".to_string(),
        Op::Load(slot) => format!("LOAD {}", slot),
        Op::Store(slot) => format!("STORE {}", slot),
        Op::Print => "PRINT".to_string(),
        Op::Halt => "HALT".to_string(),
    }
}

fn run_internal(code: &[u8], trace: bool) -> Result<(), String> {
    let mut stack = Vec::new();
    let mut globals = [0i64; 256];
    let mut ip = 0;

    while ip < code.len() {
        let current_ip = ip;
        let (op, next_ip) = Op::decode(code, ip)?;
        ip = next_ip;

        if trace {
            println!("ip={} {} stack={:?}", current_ip, format_op(op), stack);
        }

        match op {
            Op::Push(value) => {
                if stack.len() >= 1024 {
                    return Err("stack overflow".into());
                }
                stack.push(value);
            }
            Op::Pop => {
                stack.pop().ok_or_else(|| "stack underflow".to_string())?;
            }
            Op::Dup => {
                let value = *stack.last().ok_or_else(|| "stack underflow".to_string())?;
                if stack.len() >= 1024 {
                    return Err("stack overflow".into());
                }
                stack.push(value);
            }
            Op::Swap => {
                if stack.len() < 2 {
                    return Err("stack underflow".into());
                }
                let len = stack.len();
                stack.swap(len - 1, len - 2);
            }
            Op::Add => {
                let b = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                let a = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                stack.push(a + b);
            }
            Op::Sub => {
                let b = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                let a = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                stack.push(a - b);
            }
            Op::Mul => {
                let b = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                let a = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                stack.push(a * b);
            }
            Op::Div => {
                let b = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                let a = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                if b == 0 {
                    return Err("divide by zero".into());
                }
                stack.push(a / b);
            }
            Op::Mod => {
                let b = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                let a = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                if b == 0 {
                    return Err("modulo by zero".into());
                }
                stack.push(a % b);
            }
            Op::Neg => {
                let value = stack.last_mut().ok_or_else(|| "stack underflow".to_string())?;
                *value = -*value;
            }
            Op::Load(slot) => {
                if stack.len() >= 1024 {
                    return Err("stack overflow".into());
                }
                stack.push(globals[slot as usize]);
            }
            Op::Store(slot) => {
                let value = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                globals[slot as usize] = value;
            }
            Op::Print => {
                let value = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
                println!("{}", value);
            }
            Op::Halt => {
                return Ok(());
            }
        }
    }

    Err("execution reached end of bytecode without HALT".into())
}

pub fn run(code: &[u8]) -> Result<(), String> {
    run_internal(code, false)
}

pub fn run_trace(code: &[u8]) -> Result<(), String> {
    run_internal(code, true)
}

pub fn run_file(input: &str, trace: bool) -> Result<(), String> {
    let code = bytecode::read_bytecode(input)?;
    if trace {
        run_trace(&code)
    } else {
        run(&code)
    }
}
