mod assembler;
mod bytecode;
mod disassembler;
mod error;
mod isa;
mod vm;

use std::env;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(usage());
    }

    match args[1].as_str() {
        "asm" => {
            if args.len() != 5 || args[3] != "-o" {
                return Err("Usage: minivm asm <input.tasm> -o <output.tbc>".into());
            }
            assembler::assemble(&args[2], &args[4])
        }
        "run" => {
            match args.len() {
                3 => vm::run(&args[2], false),
                4 if args[2] == "--trace" => vm::run(&args[3], true),
                _ => Err("Usage: minivm run [--trace] <input.tbc>".into()),
            }
        }
        "dis" => {
            if args.len() != 3 {
                return Err("Usage: minivm dis <input.tbc>".into());
            }
            disassembler::disassemble(&args[2])
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "Usage:\n    minivm asm <input.tasm> -o <output.tbc>\n    minivm run <input.tbc>\n    minivm run --trace <input.tbc>\n    minivm dis <input.tbc>\n".into()
}
