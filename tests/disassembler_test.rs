#![allow(unused_variables, non_snake_case)]
mod parsing;

use std::fmt::Write as FmtWrite;
use std::io::Write as IOWrite;
use std::process::{Command, Stdio};
use std::str::FromStr;

use itertools::Itertools;
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

struct AsmGenerator {
    op: String,
    num_regs: usize,
    suffix: bool,
    shift: bool,
    shift_reg: bool,
    modified_imm_value: bool,
    imm_value: Option<(u32, u32)>,
}

impl AsmGenerator {
    fn new(op: &str) -> Self {
        Self {
            op: op.into(),
            num_regs: 0,
            suffix: true,
            shift: false,
            shift_reg: false,
            modified_imm_value: false,
            imm_value: None,
        }
    }

    fn no_suffix(&mut self) -> &mut Self {
        self.suffix = false;
        self
    }

    fn register(&mut self) -> &mut Self {
        self.num_regs += 1;
        self
    }

    fn modified_immediate_value(&mut self) -> &mut Self {
        self.modified_imm_value = true;
        self
    }

    fn immediate_value(&mut self, min: u32, max: u32) -> &mut Self {
        self.imm_value = Some((min, max));
        self
    }

    fn reg_shift(&mut self) -> &mut Self {
        self.shift = true;
        self.shift_reg = true;
        self
    }

    fn imm_shift(&mut self) -> &mut Self {
        self.shift = true;
        self
    }

    fn generate(&self) -> String {
        let mut rng = thread_rng();
        let mut input = String::new();

        if self.num_regs > 3 {
            panic!("Too many register combinations!");
        }

        for comb in (0..self.num_regs)
            .map(|_| &REG_OPTS)
            .multi_cartesian_product()
        {
            let mut line = String::new();
            let op = &self.op;
            let cond = COND_OPTS.choose(&mut rng).unwrap();
            let s = if self.suffix {
                S_OPTS.choose(&mut rng).unwrap()
            } else {
                ""
            };
            let regs = comb.iter().map(|s| *s).join(", ");
            write!(line, "{op}{cond}{s} {regs}").unwrap();

            if self.shift {
                if self.shift_reg {
                    let shift = SHIFT_OPS[..4].choose(&mut rng).unwrap();
                    let r = REG_OPTS.choose(&mut rng).unwrap();
                    write!(line, ", {shift} {r}").unwrap();
                } else {
                    let shift = Self::gen_random_shift();
                    if shift != "" {
                        write!(line, ", {shift}").unwrap();
                    }
                }
            } else if self.modified_imm_value {
                let imm = Self::gen_modified_imm_value();
                write!(line, ", #{imm}").unwrap();
            } else if self.imm_value.is_some() {
                let bounds = self.imm_value.unwrap();
                let imm: u32 = rng.gen_range(bounds.0..bounds.1);
                write!(line, ", #{imm}").unwrap();
            }

            writeln!(input, "{line}").unwrap();
        }
        input
    }

    /// 12-bit ARM constants use a split 8-bit base and 4-bit rotation encoding to provide a larger
    /// range of values
    fn gen_modified_imm_value() -> u32 {
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
                write!(res, "{}", shift).unwrap();
                return res;
            }
            _ => {
                return res;
            }
        };
        write!(res, "{} #{}", shift, imm).unwrap();
        res
    }
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
            "--rm",
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
fn disassembler_test_case(input: &str) {
    let output = gas_assemble_input(input.to_string());
    for line in output.lines() {
        match AsmLine::from_str(line) {
            Ok(asm_line) => {
                assert_eq!(
                    disassemble_arm(asm_line.encoding).unwrap(),
                    asm_line.instr,
                    "Disassembling {line}",
                )
            }
            Err(err) => {
                assert_eq!(err, ParseError::FormatError, "Failed to parse line \"{line}\"");
            }
        }
    }
}

#[rstest]
fn test_disasm_data_proc_instr_reg_shift(
    #[values("AND", "EOR", "SUB", "RSB", "ADD", "ADC", "SBC", "RSC", "ORR", "BIC")] op: &str,
) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .register()
            .register()
            .register()
            .reg_shift()
            .generate(),
    );
}

#[rstest]
fn test_disasm_data_proc_instr_imm_shift(
    #[values("AND", "EOR", "SUB", "RSB", "ADD", "ADC", "SBC", "RSC", "ORR", "BIC")] op: &str,
) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .register()
            .register()
            .register()
            .imm_shift()
            .generate(),
    );
}

#[rstest]
fn test_disasm_data_proc_instr_imm(
    #[values("AND", "EOR", "SUB", "RSB", "ADD", "ADC", "SBC", "RSC", "ORR", "BIC")] op: &str,
) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .register()
            .register()
            .modified_immediate_value()
            .generate(),
    );
}

#[rstest]
fn test_disasm_comparison_instr_reg_shift(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .no_suffix()
            .register()
            .register()
            .reg_shift()
            .generate(),
    );
}

#[rstest]
fn test_disasm_comparison_instr_imm_shift(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .no_suffix()
            .register()
            .register()
            .imm_shift()
            .generate(),
    );
}

#[rstest]
fn test_disasm_comparison_instr_imm(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .no_suffix()
            .register()
            .modified_immediate_value()
            .generate(),
    );
}

#[rstest]
fn test_disasm_shift_instr_imm(#[values("LSL", "LSR", "ASR", "ROR")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .register()
            .register()
            // If imm value is 0, get MOV instead
            .immediate_value(1, 32)
            .generate(),
    );
}

#[rstest]
fn test_disasm_shift_instr_reg(#[values("LSL", "LSR", "ASR", "ROR")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .register()
            .register()
            .register()
            .generate(),
    );
}

#[rstest]
fn test_disasm_instr_MOV() {
    disassembler_test_case(&AsmGenerator::new("MOV").register().register().generate());
    disassembler_test_case(
        &AsmGenerator::new("MOV")
            .register()
            .modified_immediate_value()
            .generate(),
    );
}

#[rstest]
fn test_disasm_instr_RRX() {
    // Assembler complains about RRX<cond> without S flag. Probably overlaps with ROR or something?
    // disassembler_test_case(&AsmGenerator::new("RRX").register().register().generate());
}
