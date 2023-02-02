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

/// A set of options used to automatically generate assembly instructions for test cases.
/// Instructions are generated to cover all possible combinations of register arguments, along with
/// randomized selections for other paramters (cond, set_flags, immediate values, etc.)
struct AsmGenerator {
    op: String,
    num_regs: usize,
    cond: bool,
    suffix: bool,
    shift: bool,
    shift_reg: bool,
    modified_imm_value: bool,
    imm_value: Option<(u32, u32)>,
    start_addr: Option<usize>,
    end_addr: Option<usize>,
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
            start_addr: None,
            end_addr: None,
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

    /// [min, max)
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
        self.start_addr = Some(self.num_regs);
        self
    }

    fn end_addr(&mut self) -> &mut Self {
        self.end_addr = match self.imm_value {
            Some(_) => Some(self.num_regs + 1),
            None => Some(self.num_regs),
        };
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
                write!(line, ".syntax unified; ").unwrap();
            }

            write!(line, "{op}{cond}{s}").unwrap();

            for (i, reg) in comb.iter().enumerate() {
                if self.start_addr.is_some() && self.start_addr.unwrap() == i {
                    write!(line, " [").unwrap();
                }
                write!(line, " {reg}").unwrap();

                if self.end_addr.is_some() && i + 1 == self.end_addr.unwrap() {
                    let excl =
                        if self.num_regs == self.end_addr.unwrap() && self.imm_value.is_none() {
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
                if self.end_addr.is_some() && (self.end_addr.unwrap() > self.num_regs) {
                    write!(line, " ]").unwrap();
                }
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
                    disassemble_arm(asm_line.encoding),
                    Ok(asm_line.instr),
                    "Disassembling {line}",
                )
            }
            Err(err) => {
                if let ParseError::Failure { msg } = &err {
                    println!("{}", msg);
                }
                assert_eq!(err, ParseError::NotParsed, "Failed to parse line \"{line}\"");
            }
        }
    }
}

/// Parses the gas listing, passing the encoding to the disassembler and compares the output against
/// the instruction parsed from the assembly
fn disassembler_test_case(input: &str) {
    let output = gas_assemble_input(input.to_string());

    let mut f = std::fs::File::create("test_in.s").unwrap();
    f.write_all(&input.as_bytes()).unwrap();

    let mut f = std::fs::File::create("test_out.s").unwrap();
    f.write_all(&output.as_bytes()).unwrap();

    disassemble_and_compare(&output);
}

#[rstest]
fn test_disasm_data_proc(
    #[values("AND", "EOR", "SUB", "RSB", "ADD", "ADC", "SBC", "RSC", "ORR", "BIC")] op: &str,
) {
    let reg_shift = AsmGenerator::new(op)
        .register()
        .register()
        .register()
        .reg_shift()
        .generate();
    let imm_shift = AsmGenerator::new(op)
        .register()
        .register()
        .register()
        .imm_shift()
        .generate();
    let imm = AsmGenerator::new(op)
        .register()
        .register()
        .modified_immediate_value()
        .generate();

    let input = reg_shift + &imm_shift + &imm;
    disassembler_test_case(&input);
}

#[rstest]
fn test_disasm_comparison(#[values("TST", "TEQ", "CMP", "CMN", "MVN")] op: &str) {
    let reg_shift = AsmGenerator::new(op)
        .no_s_suffix()
        .register()
        .register()
        .reg_shift()
        .generate();
    let imm_shift = AsmGenerator::new(op)
        .no_s_suffix()
        .register()
        .register()
        .imm_shift()
        .generate();
    let imm = AsmGenerator::new(op)
        .no_s_suffix()
        .register()
        .modified_immediate_value()
        .generate();
    let input = reg_shift + &imm_shift + &imm;
    disassembler_test_case(&input);
}

#[rstest]
fn test_disasm_shift_imm(#[values("LSL", "LSR", "ASR", "ROR")] op: &str) {
    let imm_shift = AsmGenerator::new(op)
        .register()
        .register()
        // If imm value is 0, get MOV instead
        .immediate_value(1, 32)
        .generate();
    let reg_shift = AsmGenerator::new(op)
        .register()
        .register()
        .register()
        .generate();
    let input = imm_shift + &reg_shift;
    disassembler_test_case(&input);
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
fn test_disasm_BX() {
    disassembler_test_case(
        &AsmGenerator::new("BX")
            .no_s_suffix()
            .no_pc()
            .register()
            .generate(),
    )
}

#[rstest]
fn test_disasm_extra_load_store(#[values("LDRH", "STRH", "LDRSB", "LDRSH")] op: &str) {
    let post_index_reg = AsmGenerator::new(op)
        .ual()
        .no_s_suffix()
        .no_pc()
        .register()
        .start_addr()
        .register()
        .end_addr()
        .register()
        .generate();
    let pre_index_reg = AsmGenerator::new(op)
        .ual()
        .no_s_suffix()
        .no_pc()
        .register()
        .start_addr()
        .register()
        .register()
        .end_addr()
        .generate();
    let post_index_imm = AsmGenerator::new(op)
        .ual()
        .no_s_suffix()
        .no_pc()
        .register()
        .start_addr()
        .register()
        .end_addr()
        .immediate_value(0, 256)
        .generate();
    let pre_index_imm = AsmGenerator::new(op)
        .ual()
        .no_s_suffix()
        .no_pc()
        .register()
        .start_addr()
        .register()
        .immediate_value(0, 256)
        .end_addr()
        .generate();
    let input = post_index_reg + &pre_index_reg + &pre_index_imm + &pre_index_reg;
    disassembler_test_case(&input);
}

#[rstest]
fn test_disasm_load_store(#[values("LDR")] op: &str) {
    let mut rng = thread_rng();
    let mut input = String::new();

    // Pre-index/Offset reg
    for comb in (0..3).map(|_| &REG_OPTS[..15]).multi_cartesian_product() {
        let cond = COND_OPTS.choose(&mut rng).unwrap();
        let excl = ["", "!"].choose(&mut rng).unwrap();
        let sign = ["", "-"].choose(&mut rng).unwrap();
        let shift = AsmGenerator::gen_random_shift();
        write!(&mut input, "{op}{cond} {}, [{}, {sign}{}", comb[0], comb[1], comb[2]).unwrap();
        if shift != "" {
            write!(&mut input, ", {shift}").unwrap();
        }
        writeln!(&mut input, "]{excl}").unwrap();
    }
    // Pre-index/Offset imm
    // Post-index reg
    // Post-index imm

    disassembler_test_case(&input);
}
