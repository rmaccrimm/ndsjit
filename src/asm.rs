mod execbuffer;
mod x64;

use super::ir::{Operand, VReg, IR};
use execbuffer::ExecBuffer;
use x64::*;

use std::collections::HashMap;

pub struct AssemblerX64 {
    code: Vec<u8>,
    reg_alloc: HashMap<VReg, RegX64>,
}

/* Planned use - these methods won't be called directly to setup machine code, but instead the
emit/translate/assemble (tbd) function will be passed IR instructions to encode, and then the
get_exec_buffer will be called, something like:
    let func = AssemblerX64::new(reg_alloc)
        .emit(&instructions)
        .get_exec_buffer();
*/
impl AssemblerX64 {
    pub fn new() -> AssemblerX64 {
        AssemblerX64 {
            code: Vec::new(),
            reg_alloc: HashMap::new(),
        }
    }

    pub fn get_exec_buffer(self) -> ExecBuffer {
        ExecBuffer::from_vec(self.code).unwrap()
    }

    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    fn gen_prologue(&mut self /*, reg_allocation: HashMap<RegArm32, RegX64>*/) -> &mut Self {
        mov_reg64_ptr64(&mut self.code, RegX64::RAX, RegX64::RCX);
        self
    }

    // Move physical register values back to virtual state (through pointer still stored in %rcx -
    // maybe should move to stack to free up another register?)
    fn gen_epilogue(&mut self /*, reg_allocation: HashMap<RegArm32, RegX64> */) -> &mut Self {
        mov_ptr64_reg64(&mut self.code, RegX64::RCX, RegX64::RAX);
        self
    }

    pub fn emit(&mut self, instr: IR) -> &mut Self {
        match instr {
            IR::MOV(op1, op2) => self.mov(op1, op2),
        }
    }

    fn mov(&mut self, dest: Operand, src: Operand) -> &mut Self {
        match (dest, src) {
            (Operand::Ptr(_), Operand::Ptr(_)) => {
                panic!("Invalid mov call. Cannot move between two memory addresses")
            }
            (Operand::Reg(d), Operand::Reg(s)) => mov_reg64_reg64(
                &mut self.code,
                *self.reg_alloc.get(&d).unwrap(),
                *self.reg_alloc.get(&s).unwrap(),
            ),
            (Operand::Reg(d), Operand::Ptr(s)) => mov_reg64_ptr64(
                &mut self.code,
                *self.reg_alloc.get(&d).unwrap(),
                *self.reg_alloc.get(&s).unwrap(),
            ),
            (Operand::Ptr(d), Operand::Reg(s)) => mov_ptr64_reg64(
                &mut self.code,
                *self.reg_alloc.get(&d).unwrap(),
                *self.reg_alloc.get(&s).unwrap(),
            ),
        };
        self
    }

    fn ret(&mut self) -> &mut Self {
        ret(&mut self.code);
        self
    }
}
