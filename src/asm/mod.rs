use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::num;

enum RegArm32 {}

#[derive(Copy, Clone)]
pub enum RegX64 {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

enum Operand {
    Reg(RegX64),
    Ptr(RegX64),
}

#[derive(Default)]
pub struct AssemblerX64 {
    pub code: Vec<u8>,
}

fn write_bytes(v: &mut Vec<u8>, bytes: &[u8]) {
    for &b in bytes {
        v.push(b);
    }
}

// Bits 3-7 are constant, bit 2 is the msb of the register field, bit 0 is the msb of the r/m field
fn rex_prefix(reg: RegX64, rm: RegX64) -> u8 {
    0x48 | ((reg as u8 >> 3) & 1) << 2 | ((rm as u8 >> 3) & 1)
}

fn mod_rm_byte(mod_: u8, reg: RegX64, rm: RegX64) -> u8 {
    assert!(mod_ < 4);
    (mod_ & 0x3) << 6 | (reg as u8 & 0x7) << 3 | (rm as u8 & 0x7)
}

// This will maybe be higher level later, with functions to assemble IR code, but for now methods
// correspond to x86_64 instructions
impl AssemblerX64 {
    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    pub fn gen_prologue(&mut self, /*, reg_allocation: HashMap<RegArm32, RegX64>*/) -> &mut Self {
        self.mov_reg64_ptr64(RegX64::RAX, RegX64::RCX)
    }

    // Move physical register values back to virtual state (through pointer still stored in %rcx -
    // maybe should move to stack to free up another register?)
    pub fn gen_epilogue(&mut self, /*, reg_allocation: HashMap<RegArm32, RegX64> */) -> &mut Self {
        self.mov_ptr64_reg64(RegX64::RCX, RegX64::RAX);
        self.ret()
    }

    pub fn add_rax_imm32(&mut self, imm: u32) -> &mut Self {
        write_bytes(
            &mut self.code,
            &[
                0x48,
                0x05,
                (imm & 0xff) as u8,
                ((imm >> 8) & 0xff) as u8,
                ((imm >> 16) & 0xff) as u8,
                ((imm >> 24) & 0xff) as u8,
            ],
        );
        self
    }

    pub fn mov_reg64_ptr64(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        write_bytes(
            &mut self.code,
            &[rex_prefix(dest, src), 0x8b, mod_rm_byte(0, dest, src)],
        );
        self
    }

    pub fn mov_ptr64_reg64(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        write_bytes(
            &mut self.code,
            &[rex_prefix(src, dest), 0x89, mod_rm_byte(0, src, dest)],
        );
        self
    }

    pub fn ret(&mut self) -> &mut Self {
        self.code.push(0xc3);
        self
    }
}
