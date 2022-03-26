mod execbuffer;
mod x64;

use execbuffer::ExecBuffer;
use x64::*;

pub enum Operand {
    Reg(RegX64),
    // Will probably include scale and offsets later
    Ptr(RegX64),
}

#[derive(Default)]
pub struct AssemblerX64 {
    pub code: Vec<u8>,
}

// This will maybe be higher level later, with functions to assemble IR code, but for now methods
// correspond to x86_64 instructions
impl AssemblerX64 {
    pub fn get_exec_buffer(self) -> ExecBuffer {
        ExecBuffer::from_vec(self.code).unwrap()
    }

    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    pub fn gen_prologue(&mut self, /*, reg_allocation: HashMap<RegArm32, RegX64>*/) -> &mut Self {
        self.mov(Operand::Reg(RegX64::RAX), Operand::Ptr(RegX64::RCX))
    }

    // Move physical register values back to virtual state (through pointer still stored in %rcx -
    // maybe should move to stack to free up another register?)
    pub fn gen_epilogue(&mut self, /*, reg_allocation: HashMap<RegArm32, RegX64> */) -> &mut Self {
        self.mov(Operand::Ptr(RegX64::RCX), Operand::Reg(RegX64::RAX))
    }

    pub fn translate()

    pub fn mov(&mut self, dest: Operand, src: Operand) -> &mut Self {
        match (dest, src) {
            (Operand::Ptr(_), Operand::Ptr(_)) => {
                panic!("Invalid mov call. Cannot move between two memory addresses")
            }
            (Operand::Reg(d), Operand::Reg(s)) => mov_reg64_reg64(&mut self.code, d, s),
            (Operand::Reg(d), Operand::Ptr(s)) => mov_reg64_ptr64(&mut self.code, d, s),
            (Operand::Ptr(d), Operand::Reg(s)) => mov_ptr64_reg64(&mut self.code, d, s),
        };
        self
    }

    pub fn ret(&mut self) -> &mut Self {
        ret(&mut self.code);
        self
    }
}

