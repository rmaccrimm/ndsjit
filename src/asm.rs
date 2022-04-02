pub mod alloc;
mod execbuffer;
mod x64;

use std::mem;

use super::ir::{Instr, Opcode::MOV, Operand, Operand::Reg, VReg};
use alloc::{MappedReg::Phys, RegAllocation};
use execbuffer::ExecBuffer;
use x64::*;

pub struct AssemblerX64 {
    pub code: EmitterX64,
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
            code: EmitterX64::new(),
            reg_alloc,
        }
    }

    pub fn get_exec_buffer(self) -> ExecBuffer {
        ExecBuffer::from_vec(self.code.get_buf()).unwrap()
    }

    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    pub fn gen_prologue(&mut self) -> &mut Self {
        self.code
            .push_reg64(RegX64::RBP)
            .mov_reg64_reg64(RegX64::RBP, RegX64::RSP);
        for (i, mapping) in self.reg_alloc.mapping.iter().enumerate() {
            if let Phys(r) = mapping {
                self.code
                    .mov_reg64_ptr64_disp8(*r, RegX64::RCX, (mem::size_of::<u64>() * i) as i8);
            }
        }
        self
    }

    // Move physical register values back to virtual state (through pointer still stored in %rcx -
    // maybe should move to stack to free up another register?)
    pub fn gen_epilogue(&mut self) -> &mut Self {
        for (i, mapping) in self.reg_alloc.mapping.iter().enumerate() {
            if let Phys(r) = mapping {
                self.code
                    .mov_ptr64_reg64_disp8(RegX64::RCX, *r, (mem::size_of::<u64>() * i) as i8);
            }
        }
        self.code
            .mov_reg64_reg64(RegX64::RSP, RegX64::RBP)
            .pop_reg64(RegX64::RBP)
            .ret();
        self
    }

    pub fn emit(&mut self, instr: Instr) -> &mut Self {
        match instr.opcode {
            MOV => self.mov(instr.operands[0].unwrap(), instr.operands[1].unwrap()),
            _ => self,
        }
    }

    fn mov(&mut self, dest: Operand, src: Operand) -> &mut Self {
        match (dest, src) {
            (Reg(d), Reg(s)) => match (self.reg_alloc.get(d), self.reg_alloc.get(s)) {
                (Phys(d), Phys(s)) => {
                    dbg!(d);
                    dbg!(s);
                    self.code.mov_reg64_reg64(d, s);
                }
                _ => panic!("Unimplemented"),
            },
            _ => panic!("Unimplemented"),
        }
        self
    }

    fn ret(&mut self) -> &mut Self {
        self.code.ret();
        self
    }
}

#[cfg(test)]
mod tests {}
