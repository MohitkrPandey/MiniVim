use std::fs;
use std::io::{Read, Write};

pub const MAGIC: [u8; 4] = [0x4D, 0x56, 0x4D, 0x00];
pub const VERSION: u8 = 1;

pub fn write_bytecode(path: &str, code: &[u8]) -> Result<(), String> {
    let mut file = fs::File::create(path)
        .map_err(|err| format!("failed to create {}: {}", path, err))?;
    file.write_all(&MAGIC)
        .map_err(|err| format!("failed to write magic: {}", err))?;
    file.write_all(&[VERSION])
        .map_err(|err| format!("failed to write version: {}", err))?;
    let length = (code.len() as u32).to_le_bytes();
    file.write_all(&length)
        .map_err(|err| format!("failed to write code length: {}", err))?;
    file.write_all(code)
        .map_err(|err| format!("failed to write code: {}", err))?;
    Ok(())
}

pub fn read_bytecode(path: &str) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(path)
        .map_err(|err| format!("failed to open {}: {}", path, err))?;
    let mut header = [0u8; 9];
    file.read_exact(&mut header)
        .map_err(|err| format!("failed to read header: {}", err))?;
    if header[0..4] != MAGIC {
        return Err("invalid magic".into());
    }
    if header[4] != VERSION {
        return Err("unsupported version".into());
    }
    let length = u32::from_le_bytes([header[5], header[6], header[7], header[8]]) as usize;
    let mut code = vec![0u8; length];
    file.read_exact(&mut code)
        .map_err(|err| format!("failed to read code: {}", err))?;
    let mut extra = [0u8; 1];
    match file.read(&mut extra) {
        Ok(0) => Ok(code),
        Ok(_) => Err("extra bytes after code".into()),
        Err(err) => Err(format!("failed to read trailing bytes: {}", err)),
    }
}
