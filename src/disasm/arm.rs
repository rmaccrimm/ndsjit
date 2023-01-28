use super::armv4t::{
    AddrIndex, AddrMode, AddrOffset, Address, Cond, ExtraOperand, ImmShift, Instruction, Op,
    Operand, RegShift, Register, Shift, ShiftOp,
};
use super::bits::{bit, bit_match, bits};
use super::{DisasmError, DisasmResult};

/// Number of lookahead bytes in ARM mode
const PC_LA_ARM: u32 = 8;

/// Decode instructions described in A5.2 - Data-processing and miscellaneous instructions
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
            (_, y) if bit_match(y, "1xx1") => arm_extra_load_store(instr),
            (_, _) => Err(DisasmError::undefined(instr)),
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

pub fn arm_unconditional(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_load_store(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_media(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_branch(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_coprocessor(instr: u32) -> DisasmResult {
    todo!()
}

pub fn arm_block_data_transfer(instr: u32) -> DisasmResult {
    todo!()
}

/// Decode data-processing instructions with a register operand  that can optionally be shifted by a
///  constant amount, and shift by constant instructions
fn arm_data_proc_reg(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let imm = bits(instr, 7..11);
    let op = Op::decode_data_proc_op(op1, Some(op2), Some(imm))?;

    let mut result = Instruction::default();
    result.op = op;
    result.cond = Cond::try_from(bits(instr, 28..31))?;
    result.set_flags = bit(instr, 20) == 1;

    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    let imm5 = bits(instr, 7..11);

    let shift = ImmShift::decode(bits(instr, 5..6), imm5)?;
    let shift = (shift.imm != 0).then_some(ExtraOperand::from(shift));

    match op {
        Op::ADR => {
            return Err(DisasmError::new(
                "The second arg for ADR op cannot come from a register",
                instr,
            ));
        }
        Op::MVN => {
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rm));
            result.extra = shift;
        }
        Op::MOV | Op::RRX => {
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rm));
        }
        Op::LSL | Op::LSR | Op::ASR | Op::ROR => {
            // These instructions are actually the immediate versions, even though their encodings
            // place them in the "data-processing (register)" category
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rm));
            result.operands.push(Operand::Imm(imm5));
        }
        Op::TEQ | Op::TST | Op::CMN | Op::CMP => {
            result.operands.push(Operand::Reg(rn));
            result.operands.push(Operand::Reg(rm));
            result.extra = shift;
        }
        _ => {
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rn));
            result.operands.push(Operand::Reg(rm));
            result.extra = shift;
        }
    }
    Ok(result)
}

/// Decode data-processing instructions with a register-shifted register operand
fn arm_data_proc_shift_reg(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let op = Op::decode_data_proc_op(op1, Some(op2), None)?;

    let mut result = Instruction::default();
    result.op = op;
    result.cond = Cond::try_from(bits(instr, 28..31))?;
    result.set_flags = bit(instr, 20) == 1;

    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    let rs = Register::try_from(bits(instr, 8..11))?;
    let shift = Some(ExtraOperand::from(RegShift {
        op: match bits(instr, 5..6) {
            0b00 => ShiftOp::LSL,
            0b01 => ShiftOp::LSR,
            0b10 => ShiftOp::ASR,
            0b11 => ShiftOp::ROR,
            _ => unreachable!(),
        },
        reg: rs,
    }));

    match op {
        Op::MVN => {
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rm));
            result.extra = shift;
        }
        Op::ADR | Op::MOV | Op::RRX => {
            return Err(DisasmError::new(
                format!("{:?} op cannot be register-shifted", op).as_str(),
                instr,
            ));
        }
        // NOTE - could also encode these as a MOV
        Op::LSL | Op::LSR | Op::ASR | Op::ROR => {
            // These instructions are actually the register versions, even though their encodings
            // place them in the "data-processing (register-shifted register)" category.
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rm));
            result.operands.push(Operand::Reg(rs));
        }
        Op::TEQ | Op::TST | Op::CMN | Op::CMP => {
            result.operands.push(Operand::Reg(rn));
            result.operands.push(Operand::Reg(rm));
            result.extra = shift;
        }
        _ => {
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rn));
            result.operands.push(Operand::Reg(rm));
            result.extra = shift
        }
    }
    Ok(result)
}

/// Implements the ARMExpandImm() psuedo-code function
fn expand_imm(imm12: u32) -> u32 {
    let val = bits(imm12, 0..7);
    let shift = 2 * bits(imm12, 8..11);
    val.rotate_right(shift)
}

/// Decode data-processing instructions with an immedate data operand (excluding shift instructions)
fn arm_data_proc_imm(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op = Op::decode_data_proc_op(op1, None, None)?;

    let mut result = Instruction::default();
    result.op = op;
    result.cond = Cond::try_from(bits(instr, 28..31))?;
    result.set_flags = bit(instr, 20) == 1;

    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    let rs = Register::try_from(bits(instr, 8..11))?;
    let imm = expand_imm(bits(instr, 0..11));

    match op {
        Op::LSL | Op::LSR | Op::ASR | Op::ROR | Op::RRX => {
            return Err(DisasmError::new(
                format!("{:?} op has no immediate form", op).as_str(),
                instr,
            ));
        }
        Op::ADR | Op::MOV | Op::MVN => {
            // TODO - ADR is a PC-relative instruction. Need to figure out it the address should be
            // passed in here or if it should it should just be resolved at runtime
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Imm(imm));
        }
        Op::TEQ | Op::TST | Op::CMN | Op::CMP => {
            result.operands.push(Operand::Reg(rn));
            result.operands.push(Operand::Imm(imm));
        }
        _ => {
            result.operands.push(Operand::Reg(rd));
            result.operands.push(Operand::Reg(rn));
            result.operands.push(Operand::Imm(imm));
        }
    }
    Ok(result)
}

fn arm_misc(instr: u32) -> DisasmResult {
    let op = bits(instr, 21..22);
    let op1 = bits(instr, 16..19);
    let op2 = bits(instr, 4..6);
    let b = bit(instr, 9);

    let match_only = |x: u32, match_val: u32, ret_val: Op| {
        (x == match_val)
            .then_some(ret_val)
            .ok_or(DisasmError::undefined(instr))
    };

    let op = match (op2, b, op) {
        // Not sure yet what the special registers are. Possibly need a new enum for them?
        (0b000, 0b0, 0b00) => todo!(), // MRS
        (0b000, 0b0, 0b10) => todo!(), // MRS
        (0b000, 0b0, _) => todo!(),    // MSR
        (0b001, _, 0b01) => Op::BX,
        _ => {
            return Err(DisasmError::undefined(instr));
        }
    };

    let cond = Cond::try_from(bits(instr, 28..31))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    Ok(Instruction {
        cond,
        op,
        operands: vec![Operand::Reg(rm)],
        ..Default::default()
    })
}

fn arm_mult(instr: u32) -> DisasmResult {
    let op = match bits(instr, 21..23) {
        0b000 => Op::MUL,
        0b001 => Op::MLA,
        0b100 => Op::UMULL,
        0b101 => Op::UMLAL,
        0b110 => Op::SMULL,
        0b111 => Op::SMLAL,
        _ => {
            return Err(DisasmError::undefined(instr));
        }
    };
    let rd = Register::try_from(bits(instr, 16..19))?;
    let ra = Register::try_from(bits(instr, 12..15))?;
    let rm = Register::try_from(bits(instr, 8..11))?;
    let rn = Register::try_from(bits(instr, 0..3))?;
    let s = bit(instr, 20) == 1;

    let mut instr = Instruction::default();
    match op {
        Op::MUL => {
            instr.operands.push(Operand::Reg(rd));
            instr.operands.push(Operand::Reg(rn));
            instr.operands.push(Operand::Reg(rm));
        }
        Op::MLA | Op::UMULL | Op::SMULL | Op::SMLAL => {
            instr.operands.push(Operand::Reg(rd));
            instr.operands.push(Operand::Reg(rn));
            instr.operands.push(Operand::Reg(rm));
            instr.operands.push(Operand::Reg(ra));
        }
        _ => unreachable!(),
    }
    Ok(instr)
}

fn arm_halfword_mult(instr: u32) -> DisasmResult {
    Err(DisasmError::new("halfword multiply instructions undefined in ARMv4T", instr))
}

fn arm_sync(instr: u32) -> DisasmResult {
    let op = match bits(instr, 20..23) {
        0b0000 => Op::SWP,
        0b0100 => Op::SWPB,
        _ => {
            return Err(DisasmError::undefined(instr));
        }
    };

    let instr = Instruction::default();
    Ok(instr)
}

fn arm_extra_load_store(instr: u32) -> DisasmResult {
    let op1 = (bit(instr, 22) << 1) | (bit(instr, 20));
    let op2 = bits(instr, 5..6);

    let (op, imm) = match op2 {
        0b01 => match op1 {
            0b00 => (Op::STRH, false),
            0b01 => (Op::LDRH, false),
            0b10 => (Op::STRH, true),
            0b11 => (Op::LDRH, true),
            _ => unreachable!(),
        },
        0b10 => match op1 {
            0b01 => (Op::LDRSB, false),
            0b11 => (Op::LDRSB, true),
            _ => {
                return Err(DisasmError::undefined(instr));
            }
        },
        0b11 => match op1 {
            0b01 => (Op::LDRSH, false),
            0b11 => (Op::LDRSH, true),
            _ => {
                return Err(DisasmError::undefined(instr));
            }
        },
        _ => {
            return Err(DisasmError::undefined(instr));
        }
    };

    let rm = Register::try_from(bits(instr, 0..3))?;
    let rt = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let w = bit(instr, 21);
    let p = bit(instr, 24);
    let imm8 = (bits(instr, 8..11) << 4) | bits(instr, 0..3);

    if p == 0 && w == 1 {
        return Err(DisasmError::undefined(instr));
    }

    let index = p == 1;
    let write_back = p == 0 || w == 1;
    let mode = match (index, write_back) {
        (true, true) => AddrMode::PreIndex,
        (false, true) => AddrMode::PostIndex,
        _ => {
            return Err(DisasmError::undefined(instr));
        }
    };
    let addr = Address { base: rn, mode };
    let offset = AddrIndex { reg: rm, shift: None };

    let mut instr = Instruction::default();
    instr.operands.push(Operand::Reg(rt));
    instr.operands.push(Operand::Addr(addr));
    instr.extra = Some(offset.into());

    Ok(instr)
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
