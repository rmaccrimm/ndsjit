mod arm;
pub mod armv4t;
mod bits;
mod thumb;

use arm::*;
use armv4t::Instruction;
use bits::{bit, bits};
use std::error::Error;
use std::fmt::Display;

use self::armv4t::Cond;

#[derive(Debug)]
pub struct DisasmError {
    description: String,
    instr: u32,
}

impl DisasmError {
    fn new(description: &str, instr: u32) -> Self {
        let description = String::from(description);
        Self { description, instr }
    }

    fn unknown(instr: u32) -> Self {
        Self::new("unkown instruction", instr)
    }
}

impl Display for DisasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.description, self.instr)
    }
}

impl Error for DisasmError {}

type DisasmResult = Result<Instruction, DisasmError>;

pub fn disassemble_arm(addr: u32, instr: u32) -> DisasmResult {
    match bits(instr, 28..31) {
        0b1111 => arm_unconditional(instr),
        _ => match bits(instr, 25..27) {
            0b000 | 0b001 => arm_data_proc_and_misc(instr), // Containes some load/store as well
            0b010 => arm_load_store(instr),
            0b011 => match bit(instr, 4) {
                0 => arm_load_store(instr),
                1 => arm_media(instr),
                _ => unreachable!(),
            },
            0b100 => arm_block_data_transfer(instr),
            0b101 => arm_branch(instr),
            0b110 | 0b111 => arm_coprocessor(instr),
            _ => unreachable!(),
        },
    }
}

pub fn disassemble_thumb(addr: u32, instr: u16) -> DisasmResult {
    // TODO - there are 32-bit THUMB encodings as well? How will those be handled? Maybe it's kicked
    // back out to the binary reader which will return the extra byte? Or we just always send 2 and
    // only decode the first?
    todo!();
}
