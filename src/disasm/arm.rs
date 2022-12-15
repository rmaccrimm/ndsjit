use cranelift::codegen::ir::Inst;

use super::armv4t::Instruction;
use super::bits::{bit, bits};
use super::{DisasmError, DisasmResult};

/// Number of lookahead bytes in ARM mode
const PC_LA_ARM: u32 = 8;

/// Will cover data proc, psr, bx/blx, but also some load/store?
pub fn arm_data_proc_and_misc(instr: u32) -> DisasmResult {
    todo!()
}

// Branch with PC-relative offset
pub fn arm_branch(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_block_data_transer(instr: u32) -> DisasmResult {
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
