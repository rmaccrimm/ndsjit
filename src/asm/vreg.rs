use super::x64::{Address, EmitterX64, RegX64, EAX};
use crate::ir;

pub enum HostLoc {
    Reg(RegX64),
    Addr(Address),
}
use HostLoc::*;

pub struct VReg {
    virtual_loc: Address,
    loc: HostLoc,
}

const TEMP_REG: RegX64 = EAX;

impl VReg {
    pub fn new(reg: ir::VReg, loc: HostLoc) -> VReg {
        VReg { reg, loc }
    }

    fn mov_to_temp(&mut self, e: &mut EmitterX64) -> RegX64 {
        match self.loc {
            Reg(r) => r,
            Addr(addr) => {
                e.mov_reg_addr(TEMP_REG, addr);
                TEMP_REG
            }
        }
    }

    pub fn load(&mut self, src: &mut VReg, e: &mut EmitterX64) {
        let src = src.mov_to_temp(e);
        match self.loc {
            Reg(r) => e.mov_reg_reg(r, src),
            Addr(addr) => e.mov_addr_reg(addr, src),
        };
    }

    pub fn add(&mut self, src: &mut VReg, e: &mut EmitterX64) {
        let src = src.mov_to_temp(e);
        match self.loc {
            Reg(r) => e.add_reg_reg(r, src),
            Addr(addr) => e.add_addr_reg(addr, src),
        };
    }
}

const REG_SIZE: usize = mem::size_of::<u32>() as usize;

// Stack contains prev stack pointer, followed by spilled registers
const SPILL_START: i32 = -1 * mem::size_of::<u64>() as i32;

fn spill_stack_disp(ind: usize) -> i32 {
    SPILL_START - ((REG_SIZE * ind) as i32)
}

#[cfg(test)]
mod tests {}
