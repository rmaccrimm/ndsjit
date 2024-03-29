use super::bits::{bit, bit_match, bits, pick_bits};
use super::{DisasmError, DisasmResult};
use crate::ir::{
    AddrMode, Address, Cond, ExtraOperand, ExtraValue, ImmShift, Instruction, Offset, Op, Operand,
    Register, Shift, ShiftOp,
};

/// Number of lookahead bytes in ARM mode
const PC_LA_ARM: u32 = 8;

/// Used to decode cond from an integer
const COND_MAP: [Cond; 15] = [
    Cond::EQ,
    Cond::NE,
    Cond::CS,
    Cond::CC,
    Cond::MI,
    Cond::PL,
    Cond::VS,
    Cond::VC,
    Cond::HI,
    Cond::LS,
    Cond::GE,
    Cond::LT,
    Cond::GT,
    Cond::LE,
    Cond::AL,
];

/// Used to decode register from an integer
const REG_MAP: [Register; 16] = [
    Register::R0,
    Register::R1,
    Register::R2,
    Register::R3,
    Register::R4,
    Register::R5,
    Register::R6,
    Register::R7,
    Register::R8,
    Register::R9,
    Register::R10,
    Register::R11,
    Register::R12,
    Register::SP,
    Register::LR,
    Register::PC,
];

/// Decode instructions described in A5.2 - Data-processing and miscellaneous instructions
pub fn arm_data_proc_and_misc(instr: u32) -> DisasmResult<Instruction> {
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
            (x, y) if bit_match(y, "1xx1") => match bit(x, 2) {
                0 => arm_extra_load_store_reg(instr),
                _ => arm_extra_load_store_imm(instr),
            },
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

/// Combines the Data-processing instruction tables for register, register-shifted, and
/// immediate instruction forms. All 3 pass op1, but the rest are dependent on the instruction
/// form:
///     register args: op1, op2, imm
///     register-shifted args: op1, op2
///     immediate args: op1
pub fn decode_data_proc_op(op1: u32, op2: Option<u32>, imm: Option<u32>) -> DisasmResult<Op> {
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
            None => Op::MOV,
            Some(0b00) => match imm {
                Some(0) => Op::MOV,
                _ => Op::LSL,
            },
            Some(0b01) => Op::LSR,
            Some(0b10) => Op::ASR,
            Some(0b11) => match imm {
                Some(0) => Op::RRX,
                _ => Op::ROR,
            },
            _ => unreachable!(),
        },
        0b11100 | 0b11101 => Op::BIC,
        0b11110 | 0b11111 => Op::MVN,
        _ => {
            return Err(DisasmError::new("invalid dataproc op", op1));
        }
    };
    Ok(op)
}

/// Decode table for extra load/store operations (encoded in the data-processing instruction space).
/// Both op1 and op2 must be 2 bit values
pub fn decode_extra_load_store_op(op1: u32, op2: u32) -> DisasmResult<Op> {
    let err = Err(DisasmError::new("could not decode extra load/store op", 0));
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
                return err;
            }
        },
        0b11 => match op1 {
            0b01 => (Op::LDRSH, false),
            0b11 => (Op::LDRSH, true),
            _ => {
                return err;
            }
        },
        _ => {
            return err;
        }
    };
    Ok(op)
}

pub fn decode_load_store_op(op1: u32) -> DisasmResult<Op> {
    let op = match op1 {
        0b00010 | 0b01010 => Op::STRT,
        0b00011 | 0b01011 => Op::LDRT,
        0b00110 | 0b01110 => Op::STRBT,
        0b00111 | 0b01111 => Op::LDRBT,
        _ => match pick_bits(op1, &vec![0, 2]) {
            0b00 => Op::STR,
            0b01 => Op::LDR,
            0b10 => Op::STRB,
            _ => Op::LDRB,
        },
        _ => {
            return Err(DisasmError::new("Unrecognized load/store op", 0));
        }
    };
    Ok(op)
}

fn decode_addressing_mode(p: u32, w: u32) -> DisasmResult<AddrMode> {
    let err = Err(DisasmError::new("Invalid addressing mode", 0));
    if p == 0 && w == 1 {
        return err;
    }

    let index = p == 1;
    let write_back = p == 0 || w == 1;
    let mode = match (index, write_back) {
        (true, true) => AddrMode::PreIndex,
        (true, false) => AddrMode::Offset,
        (false, true) => AddrMode::PostIndex,
        _ => {
            return err;
        }
    };
    Ok(mode)
}

pub fn decode_imm_shift(shift_op: u32, imm5: u32) -> Result<ImmShift, DisasmError> {
    if shift_op >= 4 {
        return Err(DisasmError::new("shift op must be a 2-bit value", shift_op));
    }
    if imm5 >= 32 {
        return Err(DisasmError::new("imm shift must be a 5-bit value", imm5));
    }
    let (op, imm) = match shift_op {
        0b00 => (ShiftOp::LSL, imm5),
        0b01 => (ShiftOp::LSR, if imm5 == 0 { 32 } else { imm5 }),
        0b10 => (ShiftOp::ASR, if imm5 == 0 { 32 } else { imm5 }),
        0b11 => {
            if imm5 == 0 {
                (ShiftOp::RRX, 1)
            } else {
                (ShiftOp::ROR, imm5)
            }
        }
        _ => unreachable!(),
    };
    Ok(ImmShift { op, imm })
}

pub fn arm_unconditional(instr: u32) -> DisasmResult<Instruction> {
    todo!()
}

pub fn arm_load_store(instr: u32) -> DisasmResult<Instruction> {
    let op1 = bits(instr, 20..24);
    let op = decode_load_store_op(op1)?;
    let rn = REG_MAP[bits(instr, 16..19) as usize];
    let rt = REG_MAP[bits(instr, 12..15) as usize];

    // indicates register vs immediate
    let a = bit(instr, 25);
    let p = bit(instr, 24);
    let w = bit(instr, 21);

    let add = bit(instr, 23) == 1;
    let mode = match op {
        Op::LDRT | Op::STRT | Op::STRBT | Op::LDRBT => match a {
            1 => AddrMode::PostIndex,
            _ => AddrMode::PreIndex,
        },
        _ => decode_addressing_mode(p, w)?,
    };

    let mut instruction = Instruction {
        op,
        cond: COND_MAP[bits(instr, 28..31) as usize],
        operands: vec![Operand::Reg(rt)],
        ..Default::default()
    };

    let addr = Address { base: rn, mode };
    let offset = if a == 1 {
        let rm = REG_MAP[bits(instr, 0..3) as usize];
        let shift = decode_imm_shift(bits(instr, 5..6), bits(instr, 7..11))?;
        let shift = if shift.imm == 0 { None } else { Some(shift) };
        Offset::reg(rm, shift, add)
    } else {
        Offset::imm(bits(instr, 0..11), add)
    };
    instruction.operands.push(Operand::Addr(addr));
    instruction.extra = Some(offset.into());
    Ok(instruction)
}

pub fn arm_media(instr: u32) -> DisasmResult<Instruction> {
    Err(DisasmError::new("media instructions undefined for ARMv4", instr))
}

pub fn arm_branch(instr: u32) -> DisasmResult<Instruction> {
    let op = match bits(instr, 24..25) {
        0b10 => Op::B,
        0b11 => Op::BL,
        _ => {
            return Err(DisasmError::new("invalid branch instruction", instr));
        }
    };
    Ok(Instruction {
        cond: COND_MAP[bits(instr, 28..31) as usize],
        op: op,
        operands: vec![Operand::Imm(bits(instr, 0..23))],
        extra: None,
        set_flags: false,
    })
}

pub fn arm_coprocessor(instr: u32) -> DisasmResult<Instruction> {
    todo!()
}

pub fn arm_block_data_transfer(instr: u32) -> DisasmResult<Instruction> {
    todo!()
}

/// Decode data-processing instructions with a register operand  that can optionally be shifted by a
///  constant amount, and shift by constant instructions
fn arm_data_proc_reg(instr: u32) -> DisasmResult<Instruction> {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let imm = bits(instr, 7..11);
    let op = decode_data_proc_op(op1, Some(op2), Some(imm)).map_err(|e| e.set_instr(instr))?;

    let mut result = Instruction::default();
    result.op = op;
    result.cond = COND_MAP[bits(instr, 28..31) as usize];
    result.set_flags = bit(instr, 20) == 1;

    let rd = REG_MAP[bits(instr, 12..15) as usize];
    let rn = REG_MAP[bits(instr, 16..19) as usize];
    let rm = REG_MAP[bits(instr, 0..3) as usize];
    let imm5 = bits(instr, 7..11);

    let shift = decode_imm_shift(bits(instr, 5..6), imm5)?;
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
fn arm_data_proc_shift_reg(instr: u32) -> DisasmResult<Instruction> {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let op = decode_data_proc_op(op1, Some(op2), None).map_err(|e| e.set_instr(instr))?;

    let mut result = Instruction::default();
    result.op = op;
    result.cond = COND_MAP[bits(instr, 28..31) as usize];
    result.set_flags = bit(instr, 20) == 1;

    let rd = REG_MAP[bits(instr, 12..15) as usize];
    let rn = REG_MAP[bits(instr, 16..19) as usize];
    let rm = REG_MAP[bits(instr, 0..3) as usize];
    let rs = REG_MAP[bits(instr, 8..11) as usize];
    let shift = Some(ExtraOperand::from(Shift {
        op: match bits(instr, 5..6) {
            0b00 => ShiftOp::LSL,
            0b01 => ShiftOp::LSR,
            0b10 => ShiftOp::ASR,
            0b11 => ShiftOp::ROR,
            _ => unreachable!(),
        },
        value: ExtraValue::Reg(rs),
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
fn arm_data_proc_imm(instr: u32) -> DisasmResult<Instruction> {
    let op1 = bits(instr, 20..24);
    let op = decode_data_proc_op(op1, None, None).map_err(|e| e.set_instr(instr))?;

    let mut result = Instruction::default();
    result.op = op;
    result.cond = COND_MAP[bits(instr, 28..31) as usize];
    result.set_flags = bit(instr, 20) == 1;

    let rd = REG_MAP[bits(instr, 12..15) as usize];
    let rn = REG_MAP[bits(instr, 16..19) as usize];
    let rm = REG_MAP[bits(instr, 0..3) as usize];
    let rs = REG_MAP[bits(instr, 8..11) as usize];
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

fn arm_misc(instr: u32) -> DisasmResult<Instruction> {
    let op = bits(instr, 21..22);
    let op1 = bits(instr, 16..19);
    let op2 = bits(instr, 4..6);
    let b = bit(instr, 9);

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

    let cond = COND_MAP[bits(instr, 28..31) as usize];
    let rm = REG_MAP[bits(instr, 0..3) as usize];
    Ok(Instruction {
        cond,
        op,
        operands: vec![Operand::Reg(rm)],
        ..Default::default()
    })
}

fn arm_mult(instr: u32) -> DisasmResult<Instruction> {
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
    let rd = REG_MAP[bits(instr, 16..19) as usize];
    let ra = REG_MAP[bits(instr, 12..15) as usize];
    let rm = REG_MAP[bits(instr, 8..11) as usize];
    let rn = REG_MAP[bits(instr, 0..3) as usize];
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

fn arm_halfword_mult(instr: u32) -> DisasmResult<Instruction> {
    Err(DisasmError::new("halfword multiply instructions undefined in ARMv4T", instr))
}

fn arm_sync(instr: u32) -> DisasmResult<Instruction> {
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

fn arm_extra_load_store_reg(instr: u32) -> DisasmResult<Instruction> {
    let op1 = (bit(instr, 22) << 1) | (bit(instr, 20));
    let op2 = bits(instr, 5..6);

    let op = decode_extra_load_store_op(op1, op2).map_err(|e| e.set_instr(instr))?;

    let rm = REG_MAP[bits(instr, 0..3) as usize];
    let rt = REG_MAP[bits(instr, 12..15) as usize];
    let rn = REG_MAP[bits(instr, 16..19) as usize];
    let w = bit(instr, 21);
    let p = bit(instr, 24);
    let add = bit(instr, 23) == 1;

    let mode = decode_addressing_mode(p, w).map_err(|e| e.set_instr(instr))?;
    let addr = Address { base: rn, mode };
    let offset = Offset::reg(rm, None, add);

    Ok(Instruction {
        op,
        cond: COND_MAP[bits(instr, 28..31) as usize],
        operands: vec![Operand::Reg(rt), Operand::Addr(addr)],
        extra: Some(offset.into()),
        set_flags: false,
    })
}

fn arm_extra_load_store_imm(instr: u32) -> DisasmResult<Instruction> {
    let op1 = (bit(instr, 22) << 1) | (bit(instr, 20));
    let op2 = bits(instr, 5..6);

    let op = decode_extra_load_store_op(op1, op2).map_err(|e| e.set_instr(instr))?;

    let rt = REG_MAP[bits(instr, 12..15) as usize];
    let rn = REG_MAP[bits(instr, 16..19) as usize];
    let imm = (bits(instr, 8..11) << 4) | bits(instr, 0..3);
    let w = bit(instr, 21);
    let p = bit(instr, 24);
    let add = bit(instr, 23) == 1;

    let mode = decode_addressing_mode(p, w).map_err(|e| e.set_instr(instr))?;

    let addr = Address { base: rn, mode };
    let offset = Offset::imm(imm, add);

    Ok(Instruction {
        op,
        cond: COND_MAP[bits(instr, 28..31) as usize],
        operands: vec![Operand::Reg(rt), Operand::Addr(addr)],
        extra: Some(offset.into()),
        set_flags: false,
    })
}

fn arm_msr_and_hints(instr: u32) -> DisasmResult<Instruction> {
    todo!()
}

fn arm_load_halfword_imm(instr: u32) -> DisasmResult<Instruction> {
    todo!()
}

fn arm_load_high_halfword_imm(instr: u32) -> DisasmResult<Instruction> {
    todo!()
}
