use std::fs;
use std::io::Write;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_minivm")
}

fn tmp(suffix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut p = std::env::temp_dir();
    p.push(format!("minivm_test_{}_{}", n, suffix));
    p.to_string_lossy().into_owned()
}

fn write_file(path: &str, content: &[u8]) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content).unwrap();
}

fn cmd_asm(src: &str, out: &str) -> Output {
    Command::new(bin())
        .args(["asm", src, "-o", out])
        .output()
        .unwrap()
}

fn cmd_run(tbc: &str) -> Output {
    Command::new(bin()).args(["run", tbc]).output().unwrap()
}

fn cmd_dis(tbc: &str) -> Output {
    Command::new(bin()).args(["dis", tbc]).output().unwrap()
}

fn run_source(source: &str) -> Output {
    let src = tmp("src.tasm");
    let tbc = tmp("run.tbc");
    write_file(&src, source.as_bytes());
    let asm = cmd_asm(&src, &tbc);
    assert!(
        asm.status.success(),
        "Assembly failed: {}",
        String::from_utf8_lossy(&asm.stderr)
    );
    let result = cmd_run(&tbc);
    let _ = fs::remove_file(&src);
    let _ = fs::remove_file(&tbc);
    result
}

fn make_tbc(code: &[u8]) -> Vec<u8> {
    let mut data = vec![0x4D, 0x56, 0x4D, 0x00, 0x01];
    data.extend_from_slice(&(code.len() as u32).to_le_bytes());
    data.extend_from_slice(code);
    data
}

fn expect_trap(source: &str, expected: &str) {
    let out = run_source(source);
    assert!(
        !out.status.success(),
        "Expected trap but process succeeded"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(expected),
        "Expected '{}' in stderr, got: {}",
        expected,
        stderr
    );
}

// ─── Assembler ───────────────────────────────────────────────────────────────

#[test]
fn asm_valid_program() {
    let src = tmp("valid.tasm");
    let out = tmp("valid.tbc");
    write_file(&src, b"PUSH 7\nPUSH 3\nADD\nPRINT\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(result.status.success());
    assert!(fs::metadata(&out).is_ok());
    let _ = fs::remove_file(&src);
    let _ = fs::remove_file(&out);
}

#[test]
fn asm_unknown_instruction() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"FOOBAR\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown instruction"));
    let _ = fs::remove_file(&src);
}

#[test]
fn asm_push_missing_operand() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"PUSH\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("missing operand for PUSH")
    );
    let _ = fs::remove_file(&src);
}

#[test]
fn asm_push_invalid_value() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"PUSH abc\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("invalid integer for PUSH")
    );
    let _ = fs::remove_file(&src);
}

#[test]
fn asm_load_missing_operand() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"LOAD\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("missing operand for LOAD")
    );
    let _ = fs::remove_file(&src);
}

#[test]
fn asm_load_invalid_slot() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"LOAD 999\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("invalid slot for LOAD")
    );
    let _ = fs::remove_file(&src);
}

#[test]
fn asm_store_missing_operand() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"STORE\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("missing operand for STORE")
    );
    let _ = fs::remove_file(&src);
}

#[test]
fn asm_store_invalid_slot() {
    let src = tmp("bad.tasm");
    let out = tmp("bad.tbc");
    write_file(&src, b"STORE 256\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("invalid slot for STORE")
    );
    let _ = fs::remove_file(&src);
}

// ─── Bytecode ────────────────────────────────────────────────────────────────

#[test]
fn bc_write_and_read() {
    let src = tmp("prog.tasm");
    let out = tmp("prog.tbc");
    write_file(&src, b"PUSH 1\nHALT\n");
    let result = cmd_asm(&src, &out);
    assert!(result.status.success());
    let bytes = fs::read(&out).unwrap();
    assert_eq!(&bytes[0..4], &[0x4D, 0x56, 0x4D, 0x00], "magic mismatch");
    assert_eq!(bytes[4], 1, "version mismatch");
    let len = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
    assert_eq!(bytes.len(), 9 + len, "length field mismatch");
    let _ = fs::remove_file(&src);
    let _ = fs::remove_file(&out);
}

#[test]
fn bc_read_valid_manual() {
    let halt_code = vec![0xFF];
    let data = make_tbc(&halt_code);
    let path = tmp("manual.tbc");
    write_file(&path, &data);
    let result = cmd_run(&path);
    assert!(result.status.success());
    let _ = fs::remove_file(&path);
}

#[test]
fn bc_invalid_magic() {
    let mut data = make_tbc(&[0xFF]);
    data[0] = 0xAA;
    let path = tmp("bad_magic.tbc");
    write_file(&path, &data);
    let result = cmd_run(&path);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("invalid magic")
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn bc_invalid_version() {
    let mut data = make_tbc(&[0xFF]);
    data[4] = 0x99;
    let path = tmp("bad_ver.tbc");
    write_file(&path, &data);
    let result = cmd_run(&path);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unsupported version")
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn bc_truncated_header() {
    let data = vec![0x4D, 0x56, 0x4D, 0x00];
    let path = tmp("trunc_hdr.tbc");
    write_file(&path, &data);
    let result = cmd_run(&path);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("failed to read header")
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn bc_truncated_code() {
    let mut data = vec![0x4D, 0x56, 0x4D, 0x00, 0x01];
    data.extend_from_slice(&10u32.to_le_bytes());
    data.extend_from_slice(&[0xFF, 0x01]);
    let path = tmp("trunc_code.tbc");
    write_file(&path, &data);
    let result = cmd_run(&path);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("failed to read code")
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn bc_extra_bytes() {
    let mut data = make_tbc(&[0xFF]);
    data.push(0x00);
    let path = tmp("extra.tbc");
    write_file(&path, &data);
    let result = cmd_run(&path);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("extra bytes after code")
    );
    let _ = fs::remove_file(&path);
}

// ─── VM instruction tests ─────────────────────────────────────────────────────

#[test]
fn vm_push() {
    let out = run_source("PUSH 42\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}

#[test]
fn vm_pop() {
    let out = run_source("PUSH 1\nPUSH 2\nPOP\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

#[test]
fn vm_dup() {
    let out = run_source("PUSH 5\nDUP\nADD\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n");
}

#[test]
fn vm_swap() {
    let out = run_source("PUSH 10\nPUSH 3\nSWAP\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n");
}

#[test]
fn vm_add() {
    let out = run_source("PUSH 3\nPUSH 4\nADD\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}

#[test]
fn vm_sub() {
    let out = run_source("PUSH 10\nPUSH 3\nSUB\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}

#[test]
fn vm_mul() {
    let out = run_source("PUSH 6\nPUSH 7\nMUL\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}

#[test]
fn vm_div() {
    let out = run_source("PUSH 10\nPUSH 2\nDIV\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

#[test]
fn vm_mod() {
    let out = run_source("PUSH 10\nPUSH 3\nMOD\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

#[test]
fn vm_neg() {
    let out = run_source("PUSH 5\nNEG\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "-5\n");
}

#[test]
fn vm_load_default_zero() {
    let out = run_source("LOAD 0\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n");
}

#[test]
fn vm_store_and_load() {
    let out = run_source("PUSH 99\nSTORE 0\nLOAD 0\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "99\n");
}

#[test]
fn vm_print() {
    let out = run_source("PUSH 123\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "123\n");
}

#[test]
fn vm_halt_stops_execution() {
    let out = run_source("HALT\nPUSH 999\nPRINT\nHALT\n");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

// ─── Trap tests ───────────────────────────────────────────────────────────────

#[test]
fn trap_stack_underflow_pop() {
    expect_trap("POP\nHALT\n", "trap at ip=0x0000: stack underflow");
}

#[test]
fn trap_stack_underflow_dup() {
    expect_trap("DUP\nHALT\n", "trap at ip=0x0000: stack underflow");
}

#[test]
fn trap_stack_underflow_swap() {
    expect_trap("PUSH 1\nSWAP\nHALT\n", "stack underflow");
}

#[test]
fn trap_stack_underflow_add() {
    expect_trap("PUSH 1\nADD\nHALT\n", "stack underflow");
}

#[test]
fn trap_stack_underflow_sub() {
    expect_trap("PUSH 1\nSUB\nHALT\n", "stack underflow");
}

#[test]
fn trap_stack_underflow_mul() {
    expect_trap("PUSH 1\nMUL\nHALT\n", "stack underflow");
}

#[test]
fn trap_stack_underflow_div() {
    expect_trap("PUSH 1\nDIV\nHALT\n", "stack underflow");
}

#[test]
fn trap_stack_underflow_mod() {
    expect_trap("PUSH 1\nMOD\nHALT\n", "stack underflow");
}

#[test]
fn trap_stack_underflow_neg() {
    expect_trap("NEG\nHALT\n", "trap at ip=0x0000: stack underflow");
}

#[test]
fn trap_stack_underflow_store() {
    expect_trap("STORE 0\nHALT\n", "trap at ip=0x0000: stack underflow");
}

#[test]
fn trap_stack_underflow_print() {
    expect_trap("PRINT\nHALT\n", "trap at ip=0x0000: stack underflow");
}

#[test]
fn trap_stack_overflow_push() {
    let mut source = String::new();
    for _ in 0..1025 {
        source.push_str("PUSH 1\n");
    }
    source.push_str("HALT\n");
    expect_trap(&source, "stack overflow");
}

#[test]
fn trap_stack_overflow_dup() {
    let mut source = String::new();
    for _ in 0..1024 {
        source.push_str("PUSH 1\n");
    }
    source.push_str("DUP\nHALT\n");
    expect_trap(&source, "stack overflow");
}

#[test]
fn trap_stack_overflow_load() {
    let mut source = String::new();
    for _ in 0..1024 {
        source.push_str("PUSH 1\n");
    }
    source.push_str("LOAD 0\nHALT\n");
    expect_trap(&source, "stack overflow");
}

#[test]
fn trap_divide_by_zero() {
    expect_trap(
        "PUSH 5\nPUSH 0\nDIV\nHALT\n",
        "trap at ip=0x0012: divide by zero",
    );
}

#[test]
fn trap_modulo_by_zero() {
    expect_trap(
        "PUSH 5\nPUSH 0\nMOD\nHALT\n",
        "trap at ip=0x0012: modulo by zero",
    );
}

#[test]
fn trap_missing_halt() {
    expect_trap("PUSH 5\nPRINT\n", "missing HALT");
}

// ─── Disassembler ─────────────────────────────────────────────────────────────

#[test]
fn dis_output_format() {
    let source = "PUSH 7\nPUSH 3\nADD\nPRINT\nHALT\n";
    let src = tmp("fmt.tasm");
    let tbc = tmp("fmt.tbc");
    write_file(&src, source.as_bytes());
    let asm = cmd_asm(&src, &tbc);
    assert!(asm.status.success());
    let out = cmd_dis(&tbc);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "PUSH 7\nPUSH 3\nADD\nPRINT\nHALT\n"
    );
    let _ = fs::remove_file(&src);
    let _ = fs::remove_file(&tbc);
}

#[test]
fn dis_load_store_format() {
    let source = "PUSH 42\nSTORE 12\nLOAD 5\nPRINT\nHALT\n";
    let src = tmp("ls.tasm");
    let tbc = tmp("ls.tbc");
    write_file(&src, source.as_bytes());
    let asm = cmd_asm(&src, &tbc);
    assert!(asm.status.success());
    let out = cmd_dis(&tbc);
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("LOAD 5\n"));
    assert!(text.contains("STORE 12\n"));
    let _ = fs::remove_file(&src);
    let _ = fs::remove_file(&tbc);
}

#[test]
fn dis_round_trip_simple() {
    let source = "PUSH 7\nPUSH 3\nADD\nPRINT\nHALT\n";
    let src1 = tmp("rt1.tasm");
    let tbc1 = tmp("rt1.tbc");
    write_file(&src1, source.as_bytes());
    let r = cmd_asm(&src1, &tbc1);
    assert!(r.status.success());

    let dis_out = cmd_dis(&tbc1);
    assert!(dis_out.status.success());

    let src2 = tmp("rt2.tasm");
    let tbc2 = tmp("rt2.tbc");
    write_file(&src2, &dis_out.stdout);
    let r2 = cmd_asm(&src2, &tbc2);
    assert!(r2.status.success());

    let bytes1 = fs::read(&tbc1).unwrap();
    let bytes2 = fs::read(&tbc2).unwrap();
    assert_eq!(bytes1, bytes2, "Bytecodes differ after round-trip");

    let _ = fs::remove_file(&src1);
    let _ = fs::remove_file(&tbc1);
    let _ = fs::remove_file(&src2);
    let _ = fs::remove_file(&tbc2);
}

#[test]
fn dis_round_trip_all_ops() {
    let source = concat!(
        "PUSH 10\n",
        "PUSH 3\n",
        "DUP\n",
        "ADD\n",
        "SUB\n",
        "PUSH 2\n",
        "MUL\n",
        "PUSH 2\n",
        "DIV\n",
        "PUSH 3\n",
        "MOD\n",
        "NEG\n",
        "STORE 5\n",
        "LOAD 5\n",
        "POP\n",
        "PUSH 42\n",
        "PUSH 1\n",
        "SWAP\n",
        "PRINT\n",
        "HALT\n",
    );
    let src1 = tmp("all1.tasm");
    let tbc1 = tmp("all1.tbc");
    write_file(&src1, source.as_bytes());
    let r = cmd_asm(&src1, &tbc1);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));

    let dis_out = cmd_dis(&tbc1);
    assert!(dis_out.status.success());

    let src2 = tmp("all2.tasm");
    let tbc2 = tmp("all2.tbc");
    write_file(&src2, &dis_out.stdout);
    let r2 = cmd_asm(&src2, &tbc2);
    assert!(
        r2.status.success(),
        "{}",
        String::from_utf8_lossy(&r2.stderr)
    );

    let bytes1 = fs::read(&tbc1).unwrap();
    let bytes2 = fs::read(&tbc2).unwrap();
    assert_eq!(bytes1, bytes2, "Bytecodes differ after round-trip");

    let _ = fs::remove_file(&src1);
    let _ = fs::remove_file(&tbc1);
    let _ = fs::remove_file(&src2);
    let _ = fs::remove_file(&tbc2);
}
