use super::TranslationError;
use crate::disasm::armv4t::{
    Cond, ExtraOperand, ExtraValue, ImmShift, Instruction, Offset, Op, Operand, Register, Shift,
    ShiftOp,
};
use cranelift::prelude::{
    types::{I32, I64},
    InstBuilder, IntCC, Value,
};
use cranelift_frontend::{FunctionBuilder, Variable};

/// Maybe this will persist between block translations and store the output functions?
pub struct TranslationState {
    pub register_vars: Vec<Variable>,
}

impl TranslationState {
    pub fn get_var(&self, reg: Register) -> Variable {
        self.register_vars[reg as usize]
    }
}

pub fn translate_instruction(
    instr: &Instruction,
    state: &TranslationState,
    builder: &mut FunctionBuilder,
) -> Result<(), TranslationError> {
    match instr.cond {
        Cond::AL => {
            translate_op(instr, &state, builder)?;
        }
        _ => {
            let instr_block = builder.create_block();
            let next_block = builder.create_block();
            let res = translate_cond(instr.cond, state, builder);
            builder.ins().brif(res, instr_block, &[], next_block, &[]);

            builder.seal_block(instr_block);
            builder.switch_to_block(instr_block);

            translate_op(instr, state, builder)?;
            builder.ins().jump(next_block, &[]);

            builder.seal_block(next_block);
            builder.switch_to_block(next_block);
        }
    }
    Ok(())
}

// Returns the final result of the cond comparison
pub fn translate_cond(
    cond: Cond,
    state: &TranslationState,
    builder: &mut FunctionBuilder,
) -> Value {
    let flags = builder.use_var(state.get_var(Register::FLAGS));
    let v = 28;
    let c = 29;
    let z = 30;
    let n = 31;
    match cond {
        Cond::EQ => {
            let tmp = builder.ins().ushr_imm(flags, z);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 1)
        }
        Cond::NE => {
            let tmp = builder.ins().ushr_imm(flags, z);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 0)
        }
        Cond::CS => {
            let tmp = builder.ins().ushr_imm(flags, c);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 1)
        }
        Cond::CC => {
            let tmp = builder.ins().ushr_imm(flags, c);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 0)
        }
        Cond::MI => {
            let tmp = builder.ins().ushr_imm(flags, n);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 1)
        }
        Cond::PL => {
            let tmp = builder.ins().ushr_imm(flags, n);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 0)
        }
        Cond::VS => {
            let tmp = builder.ins().ushr_imm(flags, v);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 1)
        }
        Cond::VC => {
            let tmp = builder.ins().ushr_imm(flags, v);
            let arg = builder.ins().band_imm(tmp, 1);
            builder.ins().icmp_imm(IntCC::Equal, arg, 0)
        }
        Cond::HI => {
            let tmp = builder.ins().ushr_imm(flags, c);
            // Keep both Z(b30) and C (b29)
            let arg = builder.ins().band_imm(tmp, 3);
            // Z == 0 and C == 1
            builder.ins().icmp_imm(IntCC::Equal, arg, 1)
        }
        Cond::LS => {
            let v1 = builder.ins().ushr_imm(flags, c);
            let v2 = builder.ins().band_imm(v1, 1);
            let v3 = builder.ins().icmp_imm(IntCC::Equal, v2, 0);
            let v4 = builder.ins().ushr_imm(flags, z);
            let v5 = builder.ins().band_imm(v4, 1);
            let v6 = builder.ins().icmp_imm(IntCC::Equal, v5, 1);
            // Z == 1 or C == 0
            builder.ins().bor(v3, v6)
        }
        Cond::GE => {
            // N == V
            let v1 = builder.ins().ushr_imm(flags, n);
            let v2 = builder.ins().band_imm(v1, 1);
            let v3 = builder.ins().ushr_imm(flags, v);
            let v4 = builder.ins().band_imm(v3, 1);
            builder.ins().icmp(IntCC::Equal, v2, v4)
        }
        Cond::LT => {
            // N != V
            let v1 = builder.ins().ushr_imm(flags, n);
            let v2 = builder.ins().band_imm(v1, 1);
            let v3 = builder.ins().ushr_imm(flags, v);
            let v4 = builder.ins().band_imm(v3, 1);
            builder.ins().icmp(IntCC::NotEqual, v2, v4)
        }
        Cond::GT => {
            // Z == 0 and N == V. Probably a more efficient way to encode this
            let v1 = builder.ins().ushr_imm(flags, n);
            let v2 = builder.ins().band_imm(v1, 1);
            let v3 = builder.ins().ushr_imm(flags, v);
            let v4 = builder.ins().band_imm(v3, 1);
            let v5 = builder.ins().icmp(IntCC::Equal, v2, v4);

            let v6 = builder.ins().ushr_imm(flags, z);
            let v7 = builder.ins().band_imm(v6, 1);
            let v8 = builder.ins().icmp_imm(IntCC::Equal, v7, 0);

            builder.ins().band(v5, v8)
        }
        Cond::LE => {
            // Z == 1 or N != V
            let v1 = builder.ins().ushr_imm(flags, n);
            let v2 = builder.ins().band_imm(v1, 1);
            let v3 = builder.ins().ushr_imm(flags, v);
            let v4 = builder.ins().band_imm(v3, 1);
            let v5 = builder.ins().icmp(IntCC::NotEqual, v2, v4);

            let v6 = builder.ins().ushr_imm(flags, z);
            let v7 = builder.ins().band_imm(v6, 1);
            let v8 = builder.ins().icmp_imm(IntCC::Equal, v7, 1);

            builder.ins().bor(v5, v8)
        }
        Cond::AL => {
            panic!("no translation needed for AL cond")
        }
    }
}

pub fn translate_op(
    instr: &Instruction,
    state: &TranslationState,
    builder: &mut FunctionBuilder,
) -> Result<(), TranslationError> {
    let invalid = TranslationError::Invalid(instr.clone());
    match instr.op {
        Op::ADD => {
            let dest = instr.operands[0];
            let op1 = instr.operands[1];
            let op2 = instr.operands[2];
            match (dest, op1, op2) {
                (Operand::Reg(r1), Operand::Reg(r2), Operand::Imm(imm)) => {
                    let v1 = builder.use_var(state.get_var(r1));
                    let v2 = builder.use_var(state.get_var(r2));
                    let const_ = builder.ins().iconst(I32, imm as i64);
                    let res = builder.ins().iadd(v2, const_);
                    builder.def_var(state.get_var(r1), res);
                }
                (Operand::Reg(r1), _, _) => {
                    return Err(invalid);
                }
                (_, _, _) => {
                    return Err(invalid);
                }
            };
        }
        Op::MOV => match (instr.operands[0], instr.operands[1]) {
            (Operand::Reg(dest), Operand::Reg(src)) => {
                let base = builder.use_var(state.get_var(src));
                let result = match instr.extra {
                    Some(extra) => match extra {
                        ExtraOperand::Shift(shift) => translate_shift(base, shift, state, builder),
                        _ => {
                            return Err(invalid);
                        }
                    },
                    None => base,
                };
                builder.def_var(state.get_var(dest), result);
            }
            (Operand::Reg(dest), Operand::Imm(imm)) => {
                let result = builder.ins().iconst(I32, imm as i64);
                builder.def_var(state.get_var(dest), result);
            }
            _ => {
                return Err(TranslationError::Invalid(instr.clone()));
            }
        },
        _ => {
            todo!();
        }
    }
    Ok(())
}

/// Applies a shift, returning the new shifted value
pub fn translate_shift(
    base: Value,
    shift: Shift,
    state: &TranslationState,
    builder: &mut FunctionBuilder,
) -> Value {
    let amt = match shift.value {
        ExtraValue::Reg(reg) => builder.use_var(state.get_var(reg)),
        ExtraValue::Imm(imm) => builder.ins().iconst(I32, imm as i64),
    };
    match shift.op {
        ShiftOp::LSL => builder.ins().ishl(base, amt),
        ShiftOp::LSR => builder.ins().ushr(base, amt),
        ShiftOp::ASR => builder.ins().sshr(base, amt),
        ShiftOp::ROR => builder.ins().rotr(base, amt),
        ShiftOp::RRX => builder.ins().rotr_imm(base, 1),
    }
}
