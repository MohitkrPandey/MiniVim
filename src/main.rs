mod assembler;
mod bytecode;
mod disassembler;
mod error;
mod isa;
mod vm;

use std::env;
use std::fs;

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
            if args.len() < 2 {
                return Err("Usage: minivm asm <input.tasm> -o <output.tbc>".into());
            }
            if args.len() != 5 {
                return Err("Usage: minivm asm <input.tasm> -o <output.tbc>".into());
            }
            if args[3] != "-o" {
                return Err("Missing -o argument".into());
            }
            let input_path = &args[2];
            let output_path = &args[4];
            if input_path.is_empty() {
                return Err("Missing input file".into());
            }
            if output_path.is_empty() {
                return Err("Missing output filename".into());
            }

            let source = fs::read_to_string(input_path)
                .map_err(|err| format!("failed to read {}: {}", input_path, err))?;
            let code = assembler::assemble(&source)?;
            bytecode::write_bytecode(output_path, &code)?;
            println!("Wrote {}", output_path);
            Ok(())
        }
        "run" => {
            match args.len() {
                3 => vm::run_file(&args[2], false),
                4 if args[2] == "--trace" => vm::run_file(&args[3], true),
                _ => Err("Usage: minivm run [--trace] <input.tbc>".into()),
            }
        }
        "dis" => {
            if args.len() != 3 {
                return Err("Usage: minivm dis <input.tbc>".into());
            }
            let code = bytecode::read_bytecode(&args[2])?;
            let asm = disassembler::disassemble(&code)?;
            print!("{}", asm);
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "Usage:\n    minivm asm <input.tasm> -o <output.tbc>\n    minivm run <input.tbc>\n    minivm run --trace <input.tbc>\n    minivm dis <input.tbc>\n".into()
}
