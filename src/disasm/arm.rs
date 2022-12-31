use cranelift::codegen::ir::Inst;

use super::armv4t::Instruction;
use super::bits::{bit, bits};
use super::{DisasmError, DisasmResult};

/// Number of lookahead bytes in ARM mode
const PC_LA_ARM: u32 = 8;

pub fn arm_data_proc_and_misc(instr: u32) -> DisasmResult {
    let op = bit(instr, 25);
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 4..7);
    match op {
        0 => match op1 {
            // 10xx0
            0b10000 | 0b10010 | 0b10100 | 0b10110 => match op2 {
                _ => todo!(),
            },
        },
        1 => match op1 {
            _ => todo!(),
        },
    }
}

pub fn arm_branch(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_block_data_transfer(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_load_store(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_unconditional(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_coprocessor(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_media(instr: u32) -> DisasmResult {
    todo!()
}
