mod arm;
pub mod armv4t;
mod bits;
mod thumb;

use arm::*;
use armv4t::Instruction;
use bits::{bit, bits};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisasmError {
    description: String,
    instr: u32,
}

impl DisasmError {
    fn new(description: &str, instr: u32) -> Self {
        let description = String::from(description);
        Self { description, instr }
    }

    fn undefined(instr: u32) -> Self {
        Self::new("undefined instruction", instr)
    }

    /// Sometimes a function doesn't have access to the whole instruction and it needs to be set
    /// after Error is returned
    fn set_instr(&self, instr: u32) -> Self {
        Self { description: self.description.clone(), instr }
    }
}

impl Display for DisasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.description, self.instr)
    }
}

impl Error for DisasmError {}

type DisasmResult<T> = Result<T, DisasmError>;

pub fn disassemble_arm(instr: u32) -> DisasmResult<Instruction> {
    match bits(instr, 28..31) {
        0b1111 => arm_unconditional(instr),
        _ => match bits(instr, 25..27) {
            0b000 | 0b001 => arm_data_proc_and_misc(instr),
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

pub fn disassemble_thumb(addr: u32, instr: u16) -> DisasmResult<Instruction> {
    // TODO - there are 32-bit THUMB encodings as well? How will those be handled? Maybe it's kicked
    // back out to the binary reader which will return the extra byte? Or we just always send 2 and
    // only decode the first?
    todo!();
}

#[cfg(test)]
mod tests {
    use super::armv4t::{Cond::*, Instruction, Op::*, Operand, Register::*};
    use super::disassemble_arm;

    #[test]
    fn test_disasm_data_proc() {
        assert_eq!(
            disassemble_arm(0x020FC00C).unwrap(),
            Instruction {
                cond: EQ,
                op: AND,
                operands: vec![Operand::Reg(R12), Operand::Reg(PC), Operand::Imm(12)],
                ..Default::default()
            }
        );
    }
}
