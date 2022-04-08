pub mod alloc;
mod execbuffer;
mod x64;

use std::mem;

use super::ir::{Instr, Opcode::*, Operand, VReg};
use alloc::{MappedReg::*, RegAllocation};
use execbuffer::ExecBuffer;
use x64::RegX64::*;
use x64::*;

pub struct AssemblerX64 {
    code: EmitterX64,
    reg_alloc: RegAllocation,
}

const REG_SIZE: i8 = mem::size_of::<u64>() as i8;

// Stack contains prev stack pointer, followed by spilled registers
const SPILL_START: i8 = -1 * mem::size_of::<u64>() as i8;

// #[no_mangle]
extern "C" fn hello_world(x: *mut u64) {
    unsafe {
        *x += 33;
    }
    println!("Hello World!")
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

    pub fn with_default_alloc() -> AssemblerX64 {
        AssemblerX64 {
            code: EmitterX64::new(),
            reg_alloc: RegAllocation::default(),
        }
    }

    pub fn get_exec_buffer(self) -> ExecBuffer {
        ExecBuffer::from_vec(self.code.buf).unwrap()
    }

    pub fn hex_dump(&self) {
        for b in self.code.buf.iter() {
            print!("{:02x}", b);
        }
        println!();
    }

    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    pub fn gen_prologue(&mut self) -> &mut Self {
        let mut stack_size = self.reg_alloc.num_spilled as i32 * mem::size_of::<u64>() as i32;
        if stack_size % 16 == 0 {
            // Ensures 16-byte stack allignment after call instructions (pushes 8 byte ret addr)
            stack_size += 8;
        }
        self.code
            .push_reg64(RBP)
            .mov_reg64_reg64(RBP, RSP)
            .sub_reg64_imm32(RSP, stack_size);
        for (i, mapping) in self.reg_alloc.mapping.iter().enumerate() {
            let vreg_disp = (mem::size_of::<u64>() * i) as i8;
            match mapping {
                Phys(r) => {
                    self.code.mov_reg64_ptr64_disp8(*r, RCX, vreg_disp);
                }
                Spill(i) => {
                    // Since prev base ptr is first on stack, add 1 to each index
                    let spill_stack_disp = mem::size_of::<u64>() as i8 * -i;
                    self.code
                        .mov_reg64_ptr64_disp8(RAX, RCX, vreg_disp)
                        .mov_ptr64_reg64_disp8(RBP, RAX, SPILL_START + spill_stack_disp);
                }
                Unmapped => (),
            };
        }
        self
    }

    // Move physical register values back to virtual state (through pointer still stored in %rcx -
    // maybe should move to stack to free up another register?)
    pub fn gen_epilogue(&mut self) -> &mut Self {
        for (i, mapping) in self.reg_alloc.mapping.iter().enumerate() {
            let vreg_disp = (mem::size_of::<u64>() * i) as i8;
            match mapping {
                Phys(r) => {
                    self.code.mov_ptr64_reg64_disp8(RCX, *r, vreg_disp);
                }
                Spill(i) => {
                    // Since prev base ptr is first on stack, add 1 to each index
                    let spill_stack_disp = mem::size_of::<u64>() as i8 * -i;
                    self.code
                        .mov_reg64_ptr64_disp8(RAX, RBP, SPILL_START + spill_stack_disp)
                        .mov_ptr64_reg64_disp8(RCX, RAX, vreg_disp);
                }
                _ => (),
            }
        }
        self.code.mov_reg64_reg64(RSP, RBP).pop_reg64(RBP).ret();
        self
    }

    pub fn call_rcx(&mut self) -> &mut Self {
        dbg!(hello_world as *const ());
        dbg!((hello_world as *const ()) as u64);
        dbg!(hello_world as u64);
        self.code.mov_reg64_imm64(RAX, hello_world as u64);
        self.code.call_reg64(RAX);
        self
    }

    pub fn emit(&mut self, instr: Instr) {
        match instr.opcode {
            MOVr(dest, src) => self.mov_reg(dest, src),
            MOVi(dest, imm) => self.mov_imm(dest, imm),
            PUSH(reg) => panic!(),
            POP(reg) => panic!(),
        };
    }

    pub fn mov_reg(&mut self, dest: VReg, src: VReg) -> &mut Self {
        match (self.reg_alloc.get(dest), self.reg_alloc.get(src)) {
            (Phys(rd), Phys(rs)) => self.code.mov_reg64_reg64(rd, rs),
            (Phys(rd), Spill(is)) => {
                self.code
                    .mov_reg64_ptr64_disp8(rd, RBP, SPILL_START + REG_SIZE * -is)
            }
            (Spill(id), Phys(rs)) => {
                self.code
                    .mov_ptr64_reg64_disp8(RBP, rs, SPILL_START + REG_SIZE * -id)
            }
            (Spill(id), Spill(is)) => self
                .code
                .mov_reg64_ptr64_disp8(RAX, RBP, SPILL_START + REG_SIZE * -is)
                .mov_ptr64_reg64_disp8(RBP, RAX, SPILL_START + REG_SIZE * -is),
            _ => panic!(),
        };
        self
    }

    pub fn mov_imm(&mut self, dest: VReg, imm: i16) -> &mut Self {
        match self.reg_alloc.get(dest) {
            Phys(rd) => self.code.mov_reg64_imm32(rd, imm as i32),
            Spill(ri) => {
                self.code
                    .mov_ptr64_imm32_disp8(RBP, imm as i32, SPILL_START + REG_SIZE * -ri)
            }
            Unmapped => panic!(),
        };
        self
    }

    pub fn push(&mut self, reg: VReg) {
        match self.reg_alloc.get(reg) {
            Phys(r) => self.code.push_reg64(r),
            Spill(i) => self.code.push_ptr64_disp8(RBP, SPILL_START + REG_SIZE * -i),
            Unmapped => panic!(),
        };
    }

    pub fn str() {}

    pub fn ldr() {}

    pub fn ret(&mut self) -> &mut Self {
        self.code.ret();
        self
    }
}

#[cfg(test)]
mod tests {}
