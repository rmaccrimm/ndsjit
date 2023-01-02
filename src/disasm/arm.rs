use super::armv4t::{Cond, Instruction, Op, Operand, Register};
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

fn get_data_proc_op(op1: u32, op2: u32, imm: Option<u32>) -> Result<Op, DisasmError> {
    let op = match op1 {
        0b00000 | 0b00001 => Op::AND,
        0b00010 | 0b00011 => Op::EOR,
        0b00100 | 0b00101 => Op::SUB,
        0b00110 | 0b00111 => Op::RSB,
        0b01000 | 0b01001 => Op::ADD,
        0b01010 | 0b01011 => Op::ADC,
        0b01100 | 0b01101 => Op::SBC,
        0b01110 | 0b01111 => Op::RSC,
        0b10001 => Op::TST,
        0b10011 => Op::TEQ,
        0b10101 => Op::CMP,
        0b10111 => Op::CMN,
        0b11000 | 0b11001 => Op::ORR,
        0b11010 | 0b11011 => match op2 {
            0b00 => match imm {
                Some(0) => Op::MOV,
                _ => Op::LSL,
            },
            0b01 => Op::LSR,
            0b10 => Op::ASR,
            0b11 => match imm {
                Some(0) => Op::RRX,
                _ => Op::ROR,
            },
        },
        0b11100 | 0b11101 => Op::BIC,
        0b11110 | 0b11111 => Op::MVN,
        _ => {
            return Err(DisasmError::new("unrecognized op", op1));
        }
    };
    Ok(op)
}

fn arm_data_proc_reg(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let imm = bits(instr, 7..11);
    let op = get_data_proc_op(op1, op2, Some(imm))?;
    let result = Instruction::default();
    result.cond = Cond::try_from(instr)?;
    result.set_flags = bit(instr, 20) == 1;
    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    // Operands are placed in order they appear in the manual assembly, e.g. ADC <Rd>, <Rn>, <Rm>
    for (i, &reg) in [rd, rn, rm].iter().enumerate() {
        result.operands[i] = Some(Operand::Reg { reg, shift: None });
    }
    Ok(result)
}

fn arm_data_proc_shift_reg(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_data_proc_imm(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_misc(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_mult(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_halfword_mult(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_sync(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_branch(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_block_data_transfer(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_load_store(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_unconditional(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_coprocessor(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_media(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_extra_load_store(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_msr_and_hints(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_load_halfword_imm(instr: u32) -> DisasmResult {
    todo!()
}

fn arm_load_high_halfword_imm(instr: u32) -> DisasmResult {
    todo!()
}
