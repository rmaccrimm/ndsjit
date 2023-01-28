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
    cond: bool,
    suffix: bool,
    shift: bool,
    shift_reg: bool,
    modified_imm_value: bool,
    imm_value: Option<(u32, u32)>,
    start_addr: usize,
    end_addr: usize,
    no_pc: bool,
    ual: bool,
}

impl AsmGenerator {
    fn new(op: &str) -> Self {
        Self {
            op: op.into(),
            num_regs: 0,
            cond: true,
            suffix: true,
            shift: false,
            shift_reg: false,
            modified_imm_value: false,
            imm_value: None,
            start_addr: 0,
            end_addr: 0,
            no_pc: false,
            ual: false,
        }
    }

    fn ual(&mut self) -> &mut Self {
        self.ual = true;
        self
    }

    fn no_s_suffix(&mut self) -> &mut Self {
        self.suffix = false;
        self
    }

    fn no_cond(&mut self) -> &mut Self {
        self.cond = false;
        self
    }

    fn no_pc(&mut self) -> &mut Self {
        self.no_pc = true;
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

    fn start_addr(&mut self) -> &mut Self {
        self.start_addr = self.num_regs;
        self
    }

    fn end_addr(&mut self) -> &mut Self {
        self.end_addr = self.num_regs;
        self
    }

    fn generate(&self) -> String {
        let mut rng = thread_rng();
        let mut input = String::new();

        if self.num_regs > 3 {
            panic!("Too many register combinations!");
        }

        let reg_end = if self.no_pc { 15 } else { 16 };

        for comb in (0..self.num_regs)
            .map(|_| &REG_OPTS[..reg_end])
            .multi_cartesian_product()
        {
            let mut line = String::new();
            let op = &self.op;
            let cond = if self.cond {
                COND_OPTS.choose(&mut rng).unwrap()
            } else {
                ""
            };
            let s = if self.suffix {
                S_OPTS.choose(&mut rng).unwrap()
            } else {
                ""
            };

            if self.ual {
                write!(line, ".syntax unified; ");
            }

            write!(line, "{op}{cond}{s}").unwrap();

            for (i, reg) in comb.iter().enumerate() {
                if self.start_addr == i {
                    write!(line, " [").unwrap();
                }
                write!(line, " {reg}").unwrap();
                if i + 1 == self.end_addr {
                    let excl = if self.num_regs == self.end_addr {
                        ["", "!"].choose(&mut rng).unwrap()
                    } else {
                        ""
                    };
                    write!(line, " ]{excl}").unwrap();
                }
                if i + 1 != self.num_regs {
                    write!(line, ",").unwrap();
                }
            }

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

    let out = String::from_utf8(output.stdout).unwrap();

    let err_str = String::from_utf8(output.stderr).unwrap();
    for line in err_str.lines() {
        println!("{}", line);
    }
    if err_str.len() > 0 {
        panic!("Failed to assemble input");
    }

    out
}

fn disassemble_and_compare(gas_output: &str) {
    for line in gas_output.lines() {
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

/// Parses the gas listing, passing the encoding to the disassembler and compares the output against
/// the instruction parsed from the assembly
fn disassembler_test_case(input: &str) {
    let output = gas_assemble_input(input.to_string());
    disassemble_and_compare(&output);
}

#[rstest]
fn test_disasm_data_proc_reg_shift(
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
fn test_disasm_data_proc_imm_shift(
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
fn test_disasm_data_proc_imm(
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
fn test_disasm_comparison_reg_shift(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .no_s_suffix()
            .register()
            .register()
            .reg_shift()
            .generate(),
    );
}

#[rstest]
fn test_disasm_comparison_imm_shift(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .no_s_suffix()
            .register()
            .register()
            .imm_shift()
            .generate(),
    );
}

#[rstest]
fn test_disasm_comparison_imm(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    disassembler_test_case(
        &AsmGenerator::new(op)
            .no_s_suffix()
            .register()
            .modified_immediate_value()
            .generate(),
    );
}

#[rstest]
fn test_disasm_shift_imm(#[values("LSL", "LSR", "ASR", "ROR")] op: &str) {
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
fn test_disasm_shift_reg(#[values("LSL", "LSR", "ASR", "ROR")] op: &str) {
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
fn test_disasm_RRX() {
    // Assembler complains about RRX<cond> without S flag. Probably overlaps with ROR or something?
    // disassembler_test_case(&AsmGenerator::new("RRX").register().register().generate());
}

#[rstest]
fn test_disasm_BX() {
    // Hard-coding this case because BX pc gives an error (but still assembles)
    let gas_output = "
        1 0000 E12FFF10      BX r0\n
        2 0004 E12FFF11      BX r1\n
        3 0008 E12FFF12      BX r2\n
        4 000c E12FFF13      BX r3\n
        5 0010 E12FFF14      BX r4\n
        6 0014 E12FFF15      BX r5\n
        7 0018 E12FFF16      BX r6\n
        8 001c E12FFF17      BX r7\n
        9 0020 E12FFF18      BX r8\n
        10 0024 E12FFF19     BX r9\n
        11 0028 E12FFF1A     BX r10\n
        12 002c E12FFF1B     BX r11\n
        13 0030 E12FFF1C     BX r12\n
        14 0034 E12FFF1D     BX sp\n
        15 0038 E12FFF1E     BX lr\n
        16 003c E12FFF1F     BX pc";
    disassemble_and_compare(&gas_output);
}

#[rstest]
fn test_disasm_mem_access() {
    disassembler_test_case(
        &AsmGenerator::new("LDRH")
            .ual()
            .no_s_suffix()
            .no_pc()
            .register()
            .start_addr()
            .register()
            .register()
            .end_addr()
            .generate(),
    );
}
