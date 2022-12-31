use super::armv4t::Instruction;
use super::bits::{bit, bit_match, bits};
use super::{DisasmError, DisasmResult};

/// Number of lookahead bytes in ARM mode
const PC_LA_ARM: u32 = 8;

pub fn arm_data_proc_and_misc(instr: u32) -> DisasmResult {
    let op = bit(instr, 25);
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 4..7);
    if op == 0 {
        match (op1, op2) {
            (x, y) if !bit_match(x, "10xx0") && bit_match(y, "xxx0") => arm_data_proc_reg(instr),
            (x, y) if !bit_match(x, "10xx0") && bit_match(y, "0xx1") => {
                arm_data_proc_shift_reg(instr)
            }
            (x, y) if bit_match(x, "10xx0") && bit_match(y, "0xxx") => arm_misc(instr),
            (x, y) if bit_match(x, "10xx0") && bit_match(y, "1xx0") => arm_halfword_mult(instr),
            (x, y) if bit_match(x, "0xxxx") && y == 0b1001 => arm_mult(instr),
            (x, y) if bit_match(x, "1xxxx") && y == 0b1001 => arm_sync(instr),
            (x, y) if !bit_match(x, "0xx1x") && y == 0b1011 => arm_extra_load_store(instr),
            (x, y) if !bit_match(x, "0xx1x") && bit_match(y, "11x1") => arm_extra_load_store(instr),
            (x, y) if bit_match(x, "0xx10") && bit_match(y, "11x1") => arm_extra_load_store(instr),
            (x, y) if bit_match(x, "0xx1x") && y == 0b1011 => arm_extra_load_store(instr),
            (x, y) if bit_match(x, "0xx11") && bit_match(y, "11x1") => arm_extra_load_store(instr),
            (_, _) => Err(DisasmError::unknown(instr)),
        }
    } else {
        match op1 {
            x if !bit_match(x, "10xx0") => arm_data_proc_imm(instr),
            0b10000 => arm_load_high_halfword_imm(instr),
            0b10100 => arm_load_halfword_imm(instr),
            _ => arm_msr_and_hints(instr),
        }
    }
}

pub fn arm_data_proc_reg(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_data_proc_shift_reg(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_data_proc_imm(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_misc(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_mult(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_halfword_mult(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_sync(instr: u32) -> DisasmResult {
    todo!()
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

pub fn arm_extra_load_store(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_msr_and_hints(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_load_halfword_imm(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_load_high_halfword_imm(instr: u32) -> DisasmResult {
    todo!()
}
