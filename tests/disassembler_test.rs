mod parsing;

use std::fmt::Write as FmtWrite;
use std::io::Write as IOWrite;
use std::process::{Command, Stdio};
use std::str::{from_utf8, FromStr};

use ndsjit::disasm::disassemble_arm;
use rand::{seq::SliceRandom, thread_rng};

use parsing::AsmLine;

// use ndsjit::disasm::disassemble_arm;
// use parsing::AsmLine;

const REG_OPTS: [&str; 16] = [
    "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "sp", "lr",
    "pc",
];

const COND_OPTS: [&str; 15] = [
    "EQ", "NE", "CS", "CC", "MI", "PL", "VS", "VC", "HI", "LS", "GE", "LT", "GT", "LE", "AL",
];

const S_OPTS: [&str; 2] = ["", "S"];

const SHIFT_OPS: [&str; 4] = ["LSL", "LSR", "ASR", "ROR"];

#[test]
fn test_and() {
    // let file = File::create("tests/asm/and.asm").unwrap();
    // let mut writer = BufWriter::new(file);

    let mut input = String::new();

    let mut rng = thread_rng();
    for r1 in REG_OPTS {
        for r2 in REG_OPTS {
            for r3 in REG_OPTS {
                let cond = COND_OPTS.choose(&mut rng).unwrap();
                let r4 = REG_OPTS.choose(&mut rng).unwrap();
                let shift = SHIFT_OPS.choose(&mut rng).unwrap();
                let s = S_OPTS.choose(&mut rng).unwrap();

                writeln!(input, "AND{cond}{s} {r1}, {r2}, {r3}, {shift} {r4}")
                    .expect("failed to write to input string")
            }
        }
    }
    let mut asm_proc = Command::new("docker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
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
            .write_all(input.as_bytes())
            .expect("failed to write to stdin")
    });

    let output = asm_proc
        .wait_with_output()
        .expect("failed to wait on process");

    for (i, line) in from_utf8(output.stdout.as_slice())
        .unwrap()
        .lines()
        .enumerate()
    {
        match AsmLine::from_str(line) {
            Ok(asm_line) => assert_eq!(disassemble_arm(asm_line.encoding).unwrap(), asm_line.instr),
            Err(err) => {
                println!("Failed to parse line \"{line}\": {err}")
            }
        }
    }
}
