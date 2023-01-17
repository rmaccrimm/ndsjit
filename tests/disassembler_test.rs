mod parsing;

use std::fmt::Write as FmtWrite;
use std::io::Write as IOWrite;
use std::process::{Command, Stdio};
use std::str::FromStr;

use ndsjit::disasm::disassemble_arm;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rstest::rstest;

use parsing::{AsmLine, ParseError};

const REG_OPTS: [&str; 16] = [
    "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "sp", "lr",
    "pc",
];

const COND_OPTS: [&str; 15] = [
    "EQ", "NE", "CS", "CC", "MI", "PL", "VS", "VC", "HI", "LS", "GE", "LT", "GT", "LE", "AL",
];

const S_OPTS: [&str; 2] = ["", "S"];

const SHIFT_OPS: [&str; 6] = ["LSL", "LSR", "ASR", "ROR", "RRX", ""];

/// Generate assembly for a 3-operand data-processing instruction, with a register-shifted 3rd
/// operand. Covers all combinations of registers for 3 register arguments, with condition,
/// shift-register, and "S" flag chosen randomly
fn gen_data_proc_shift_reg(mnemonic: &str) -> String {
    let mut input = String::new();
    let mut rng = thread_rng();
    for r1 in REG_OPTS {
        for r2 in REG_OPTS {
            for r3 in REG_OPTS {
                let cond = COND_OPTS.choose(&mut rng).unwrap();
                let r4 = REG_OPTS.choose(&mut rng).unwrap();
                let shift = SHIFT_OPS[..4].choose(&mut rng).unwrap();
                let s = S_OPTS.choose(&mut rng).unwrap();

                writeln!(input, "{mnemonic}{cond}{s} {r1}, {r2}, {r3}, {shift} {r4}")
                    .expect("failed to write to input string")
            }
        }
    }
    return input;
}

/// There are a limited number of 32-bit immediate values available due to the split 8-bit base and
/// 4-bit rotation encoding that ARM uses
fn gen_random_imm_value() -> u32 {
    let base = thread_rng().gen_range(0..256);
    let rotate = thread_rng().gen_range(0..12) * 2;
    base << rotate
}

fn gen_random_shift() -> String {
    let mut rng = thread_rng();
    let shift = *SHIFT_OPS.choose(&mut rng).unwrap();
    let mut res = String::new();
    let imm = match shift {
        "LSL" | "ROR" => rng.gen_range(1..32),
        "LSR" | "ASR" => rng.gen_range(1..33),
        "RRX" => {
            write!(res, ", {}", shift).unwrap();
            return res;
        }
        _ => {
            return res;
        }
    };
    write!(res, ", {} #{}", shift, imm).unwrap();
    res
}

/// Generate assembly for a 3-operand data-processing instruction, with an immediate value-shifted
/// 3rd operand. Covers all combinations of registers for 3 register arguments, with condition,
/// immediate value, and "S" flag chosen randomly
fn gen_data_proc_shift_imm(mnemonic: &str) -> String {
    let mut input = String::new();
    let mut rng = thread_rng();
    for r1 in REG_OPTS {
        for r2 in REG_OPTS {
            for r3 in REG_OPTS {
                let cond = COND_OPTS.choose(&mut rng).unwrap();
                let s = S_OPTS.choose(&mut rng).unwrap();
                let shift = gen_random_shift();

                writeln!(input, "{mnemonic}{cond}{s} {r1}, {r2}, {r3}{shift}")
                    .expect("failed to write to input string")
            }
        }
    }
    return input;
}

/// Spawns a new process running gas in a docker container, passing input as stdin and returning the
/// captured stdout. If the process fails (i.e. sterr is written to) the function panics.
fn gas_assemble_input(input: String) -> String {
    let mut asm_proc = Command::new("docker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args([
            "run",
            "-i",
            "--log-driver=none",
            "-a",
            "stdin",
            "-a",
            "stdout",
            "-a",
            "stderr",
            "asm",
        ])
        .spawn()
        .expect("Docker command failed to start");

    let mut stdin = asm_proc.stdin.take().expect("failed to get stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(&input.as_bytes())
            .expect("failed to write to stdin")
    });

    let output = asm_proc
        .wait_with_output()
        .expect("failed to wait on process");

    let err_str = String::from_utf8(output.stderr).unwrap();
    for line in err_str.lines() {
        println!("{}", line);
    }
    if err_str.len() > 0 {
        panic!("Failed to assemble input");
    }

    String::from_utf8(output.stdout).unwrap()
}

/// Parses the gas listing, passing the encoding to the disassembler and compares the output against
/// the instruction parsed from the assembly
fn disassemble_gas_output(output: &str) {
    for line in output.lines() {
        match AsmLine::from_str(line) {
            Ok(asm_line) => {
                assert_eq!(
                    disassemble_arm(asm_line.encoding).unwrap(),
                    asm_line.instr,
                    "Disassembling {}",
                    asm_line.encoding
                )
            }
            Err(err) => {
                assert_eq!(err, ParseError::FormatError, "Failed to parse line \"{line}\"");
            }
        }
    }
}

#[rstest]
#[case("AND")]
#[case("EOR")]
#[case("SUB")]
#[case("RSB")]
#[case("ADD")]
#[case("ADC")]
#[case("SBC")]
#[case("RSC")]
#[case("CMP")]
#[case("CMN")]
#[case("ORR")]
#[case("BIC")]
fn test_disasm_data_proc_instr(#[case] op: &str) {
    let input = gen_data_proc_shift_reg(op);
    let out = gas_assemble_input(input);
    disassemble_gas_output(&out);

    let input = gen_data_proc_shift_imm(op);
    let out = gas_assemble_input(input);
    disassemble_gas_output(&out);
}

#[rstest]
#[case("TST")]
#[case("TEQ")]
fn test_disasm_test_instr(#[case] op: &str) {}

#[rstest]
#[case("LSL")]
#[case("LSR")]
#[case("ASR")]
#[case("ROR")]
#[case("RRX")]
fn test_disasm_shift_instr(#[case] op: &str) {}
