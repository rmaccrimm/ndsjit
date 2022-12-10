use super::compiler::CompilerX64;
use super::disasm::armv4t::instruction::{Cond, DataProc, DataProcOp};

pub trait CodeGen {
    fn codegen(&self, cc: &mut CompilerX64);
}

impl CodeGen for DataProc {
    fn codegen(&self, cc: &mut CompilerX64) {
        let end = cc.label();
        if self.cond.is_some() {
            match self.cond.unwrap() {
                Cond::EQ => cc.jnz(end),
                Cond::NE => cc.jz(end),
                Cond::CS => cc.jnc(end),
                Cond::CC => cc.jc(end),
                Cond::MI => cc.jnn(end),
                Cond::PL => cc.jn(end),
                Cond::VS => cc.jnv(end),
                Cond::VC => cc.jv(end),
                // TODO - these implemented with several jumps to avoid needing more methods, but
                // might be faster to add methods using bitwise logic
                Cond::HI => {
                    // Run if C == 1 and Z == 0 by inverting conditional
                    // i.e. Skip if ~C OR ~Z
                    let start = cc.label();
                    cc.jnc(end).jz(end)
                }
                Cond::LS => cc.jnc(end).jz(end),
                Cond::GE => {
                    // Skip if N != V
                    // => (V and ~N) or (~V and N)
                    let not_v = cc.label();
                    let start = cc.label();
                    cc.jnv(not_v)
                        .jnn(end)
                        .jmp(start)
                        .bind(not_v)
                        .jn(end)
                        .bind(start)
                }
                Cond::LT => {
                    // Skip if N == V
                    // => (V and N) or (~V and ~N)
                    let not_v = cc.label();
                    let start = cc.label();
                    cc.jnv(not_v)
                        .jn(end)
                        .jmp(start)
                        .bind(not_v)
                        .jnn(end)
                        .bind(start)
                }
                Cond::GT => {
                    // Z == 0 and N == V
                    // Skip if Z or N != V
                    cc.jz(end);
                    let not_v = cc.label();
                    let start = cc.label();
                    cc.jnv(not_v)
                        .jnn(end)
                        .jmp(start)
                        .bind(not_v)
                        .jn(end)
                        .bind(start)
                }
                Cond::LE => {
                    // Z == 1 or N != V
                    // Skip if ~Z and N == V
                    let not_v = cc.label();
                    let eq = cc.label();
                    let start = cc.label();
                    cc.jnv(not_v)
                        .jnn(start) // V and ~N
                        .jmp(eq)
                        .bind(not_v)
                        .jn(start) // ~V and N
                        .bind(eq)
                        .jnz(end)
                        .bind(start)
                }
            };
        }
        match self.op {
            DataProcOp::ADD => match self.imm {
                None => {
                    let dest = cc.var_word(self.regs.dest.unwrap());
                    let rn = cc.var_word(self.regs.rn.unwrap());
                    cc.add_reg(dest, rn, self.update_flags);
                }
                Some(imm) => {
                    let dest = cc.var_word(self.regs.dest.unwrap());
                    cc.add_imm(dest, imm, self.update_flags);
                }
            },
            _ => todo!(),
        };
        cc.bind(end);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_eq() {
        // First - need to be able to set flags -> cmp
    }
}