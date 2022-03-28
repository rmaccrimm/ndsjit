mod alloc;
mod execbuffer;
mod x64;

use super::ir::{Instr, Operand, VReg};
use alloc::RegAllocation;
use execbuffer::ExecBuffer;
use x64::*;

pub struct AssemblerX64 {
    code: Vec<u8>,
    reg_alloc: RegAllocation,
}

/*
Planned use - these methods won't be called directly to setup machine code, but instead the
emit/translate/assemble (tbd) function will be passed IR instructions to encode, and then the
get_exec_buffer will be called, something like:

    let func = AssemblerX64::new(reg_alloc)
        .emit(&instructions)
        .get_exec_buffer();
*/
impl AssemblerX64 {
    pub fn new(reg_alloc: RegAllocation) -> AssemblerX64 {
        AssemblerX64 {
            code: Vec::new(),
            reg_alloc,
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

    pub fn emit(&mut self, instr: Instr) -> &mut Self {
        self
    }

    fn mov(&mut self, dest: Operand, src: Operand) -> &mut Self {
        self
    }

    fn ret(&mut self) -> &mut Self {
        ret(&mut self.code);
        self
    }
}
