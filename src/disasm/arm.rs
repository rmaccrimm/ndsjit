use super::armv4t::{Cond, ImmValue, Instruction, Op, Operand, Register, Shift, ShiftType};
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

fn decode_imm_shift(shift_type: u32, imm5: u32) -> Shift {
    match shift_type {
        0b00 => Shift::ImmShift {
            shift_type: ShiftType::LSL,
            shift_amt: ImmValue::Unsigned(imm5),
        },
        0b01 => {
            let shift_amt = if imm5 == 0 { 32 } else { imm5 };
            Shift::ImmShift {
                shift_type: ShiftType::LSR,
                shift_amt: ImmValue::Unsigned(shift_amt),
            }
        }
        0b10 => {
            let shift_amt = if imm5 == 0 { 32 } else { imm5 };
            Shift::ImmShift {
                shift_type: ShiftType::ASR,
                shift_amt: ImmValue::Unsigned(shift_amt),
            }
        }
        0b11 => {
            if imm5 == 0 {
                Shift::ImmShift {
                    shift_type: ShiftType::RRX,
                    shift_amt: ImmValue::Unsigned(1),
                }
            } else {
                Shift::ImmShift {
                    shift_type: ShiftType::ROR,
                    shift_amt: ImmValue::Unsigned(imm5),
                }
            }
        }
        _ => unreachable!(),
    }
}

fn arm_data_proc_reg(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let imm = bits(instr, 7..11);
    let op = get_data_proc_op(op1, op2, Some(imm))?;

    let result = Instruction::default();
    result.op = op;
    result.cond = Cond::try_from(instr)?;
    result.set_flags = bit(instr, 20) == 1;

    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    let imm5 = bits(instr, 7..11);
    let shift = decode_imm_shift(bits(instr, 5..6), imm5);

    match op {
        Op::MVN => {
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::shifted(rm, shift));
        }
        Op::MOV | Op::RRX => {
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::unshifted(rm));
        }
        Op::LSL | Op::LSR | Op::ASR | Op::ROR => {
            // These instructions are actually the immediate versions, even though their encodings
            // place them in the "data-processing (register)" category
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::unshifted(rn));
            result.operands[2] = Some(Operand::Imm(ImmValue::Unsigned(imm5)))
        }
        _ => {
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::unshifted(rn));
            result.operands[2] = Some(Operand::shifted(rm, shift));
        }
    }
    Ok(result)
}

fn arm_data_proc_shift_reg(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let op = get_data_proc_op(op1, op2, None)?;

    let result = Instruction::default();
    result.op = op;
    result.cond = Cond::try_from(instr)?;
    result.set_flags = bit(instr, 20) == 1;

    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let rm = Register::try_from(bits(instr, 0..3))?;
    let rs = Register::try_from(bits(instr, 8..11))?;
    let shift = Shift::RegShift {
        shift_type: match bits(instr, 5..6) {
            0b00 => ShiftType::LSL,
            0b01 => ShiftType::LSR,
            0b10 => ShiftType::ASR,
            0b11 => ShiftType::ROR,
        },
        shift_reg: rs,
    };

    match op {
        Op::MVN => {
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::shifted(rm, shift));
        }
        Op::MOV | Op::RRX => {
            return Err(DisasmError::new(
                format!("{:?} op cannot be register-shifted", op).as_str(),
                instr,
            ));
        }
        Op::LSL | Op::LSR | Op::ASR | Op::ROR => {
            // These instructions are actually the register versions, even though their encodings
            // place them in the "data-processing (register-shifted register)" category.
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::unshifted(rn));
            result.operands[2] = Some(Operand::unshifted(rm));
        }
        _ => {
            result.operands[0] = Some(Operand::unshifted(rd));
            result.operands[1] = Some(Operand::unshifted(rn));
            result.operands[2] = Some(Operand::shifted(rm, shift));
        }
    }
    Ok(result)
}

fn arm_data_proc_imm(instr: u32) -> DisasmResult {
    let op1 = bits(instr, 20..24);
    let op2 = bits(instr, 5..6);
    let imm = bits(instr, 7..11);
    let op = get_data_proc_op(op1, op2, Some(imm))?;
    let result = Instruction::default();
    result.cond = Cond::try_from(instr)?;
    result.set_flags = bit(instr, 20) == 1;
    let rd = Register::try_from(bits(instr, 12..15))?;
    let rn = Register::try_from(bits(instr, 16..19))?;
    let imm = bits(instr, 0..12);
    result.operands[0] = Some(Operand::unshifted(rd));
    result.operands[1] = Some(Operand::unshifted(rn));
    result.operands[2] = Some(Operand::Imm(ImmValue::Unsigned(imm)));
    Ok(result)
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
