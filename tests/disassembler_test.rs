#[allow(unused_variables)]
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
    shift: bool,
    shift_reg: bool,
    imm_value: bool,
}

impl AsmGenerator {
    fn new(op: &str) -> Self {
        Self {
            op: op.into(),
            num_regs: 0,
            shift: false,
            shift_reg: false,
            imm_value: false,
        }
    }

    fn register(&mut self) -> &mut Self {
        self.num_regs += 1;
        self
    }

    fn immediate_value(&mut self) -> &mut Self {
        self.imm_value = true;
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
        if self.shift && self.imm_value {
            panic!("Cannot have immediate value if shifting");
        }

        for comb in (0..self.num_regs)
            .map(|_| &REG_OPTS)
            .multi_cartesian_product()
        {
            let mut line = String::new();
            let op = &self.op;
            let cond = COND_OPTS.choose(&mut rng).unwrap();
            let s = S_OPTS.choose(&mut rng).unwrap();
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
            } else if self.imm_value {
                let imm = Self::gen_random_imm_value();
                write!(line, ", #{imm}").unwrap();
            }

            writeln!(input, "{line}").unwrap();
        }
        input
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
#[case("ORR")]
#[case("BIC")]
fn test_disasm_data_proc_instr(#[case] op: &str) {
    let input = AsmGenerator::new(op)
        .register()
        .register()
        .register()
        .reg_shift()
        .generate();
    let out = gas_assemble_input(input);
    disassemble_gas_output(&out);

    let input = AsmGenerator::new(op)
        .register()
        .register()
        .register()
        .imm_shift()
        .generate();
    let out = gas_assemble_input(input);
    disassemble_gas_output(&out);

    let input = AsmGenerator::new(op)
        .register()
        .register()
        .immediate_value()
        .generate();
    let out = gas_assemble_input(input);
    disassemble_gas_output(&out);
}

#[rstest]
#[case("TST")]
#[case("TEQ")]
#[case("CMP")]
#[case("CMN")]
fn test_disasm_compare_instr(#[case] _op: &str) {}

#[rstest]
#[case("LSL")]
#[case("LSR")]
#[case("ASR")]
#[case("ROR")]
#[case("RRX")]
fn test_disasm_shift_instr(#[case] _op: &str) {}

#[test]
fn try_itertools() {
    let output = AsmGenerator::new("AND")
        .register()
        .register()
        .imm_shift()
        .generate();
    for line in output.lines() {
        dbg!(line);
    }
}
