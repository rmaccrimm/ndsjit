use super::x64::*;
use crate::ir::VReg;
use std::vec::Vec;

pub const TEMP_REG: RegX64 = EAX;
pub const VREG_ADDR_REG: RegX64 = ECX;
pub const VMEM_ADDR_REG: RegX64 = EDX;

#[derive(Copy, Clone)]
pub struct RegMapping {
    pub virt_loc: Address,
    pub host_reg: Option<RegX64>,
}

impl RegMapping {
    pub fn new(virt_loc: Address, host_reg: Option<RegX64>) -> RegMapping {
        RegMapping { virt_loc, host_reg }
    }
}

pub struct RegAllocation {
    mapping: Vec<RegMapping>,
    num_spilled: usize,
    code: EmitterX64,
}

impl RegAllocation {
    pub fn default() -> RegAllocation {
        RegAllocation {
            mapping: vec![
                RegMapping::new(Address::disp(ECX, 4 * 0), Some(EBX)), // R0
                RegMapping::new(Address::disp(ECX, 4 * 1), Some(ESI)), // R1
                RegMapping::new(Address::disp(ECX, 4 * 2), Some(EDI)), // R2
                RegMapping::new(Address::disp(ECX, 4 * 3), Some(R8D)), // R3
                RegMapping::new(Address::disp(ECX, 4 * 4), Some(R9D)), // R4
                RegMapping::new(Address::disp(ECX, 4 * 5), Some(R10D)), // R5
                RegMapping::new(Address::disp(ECX, 4 * 6), Some(R11D)), // R6
                RegMapping::new(Address::disp(ECX, 4 * 7), Some(R12D)), // R7
                RegMapping::new(Address::disp(ECX, 4 * 8), Some(R13D)), // R8
                RegMapping::new(Address::disp(ECX, 4 * 9), Some(R14D)), // R9
                RegMapping::new(Address::disp(ECX, 4 * 10), Some(R15D)), // R10
                RegMapping::new(Address::disp(ECX, 4 * 11), None),     // R11
                RegMapping::new(Address::disp(ECX, 4 * 12), None),     // R12
                RegMapping::new(Address::disp(ECX, 4 * 13), None),     // SP
                RegMapping::new(Address::disp(ECX, 4 * 14), None),     // LR
                RegMapping::new(Address::disp(ECX, 4 * 15), None),     // PC
            ],
            num_spilled: 5,
            code: EmitterX64::new(),
        }
    }

    fn get(&self, reg: VReg) -> &RegMapping {
        &self.mapping[reg as usize]
    }

    pub fn load_virt_state(&self, reg: RegMapping) {
        if let Some(r) = reg.host_reg {
            self.code.mov_reg_addr(r, reg.virt_loc);
        }
    }

    pub fn save_virt_state(&self, reg: RegMapping) {
        if let Some(r) = reg.host_reg {
            self.code.mov_addr_reg(reg.virt_loc, r);
        }
    }

    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    pub fn gen_prologue(&mut self) -> &mut Self {
        let mut stack_size = self.num_spilled as i32 * mem::size_of::<u64>() as i32;
        if stack_size % 16 == 0 {
            // Ensures 16-byte stack allignment after call instructions (pushes 8 byte ret addr)
            stack_size += 8;
        }
        self.code.push_reg(RBP).mov_reg_reg(RBP, RSP).sub_reg_imm32(RSP, stack_size);

        for vreg in self.reg_alloc.mapping.iter() {
            vreg.load_virt_state(&mut self.code);
        }
        self
    }

    pub fn mov_to_temp(&self, vreg: VReg) -> RegX64 {
        let reg = self.get(vreg);
        match reg.host_reg {
            Some(r) => self.code.mov_reg_reg(TEMP_REG, r),
            None => self.code.mov_reg_addr(TEMP_REG, reg.virt_loc),
        };
        TEMP_REG
    }

    pub fn mov_reg(&self, dest: VReg, src: VReg) -> &Self {
        let tmp = self.mov_to_temp(src);
        let dest = self.get(dest);
        match dest.host_reg {
            Some(r) => self.code.mov_reg_reg(r, tmp),
            None => self.code.mov_addr_reg(dest.virt_loc, tmp),
        };
        self
    }

    pub fn mov_offset(&self, dest: VReg, base: VReg, offset: i32) -> &Self {
        let tmp = self.mov_to_temp(base);
        let dest = self.get(dest);
        let addr = Address::sib(1, tmp, VMEM_ADDR_REG, offset);
        match dest.host_reg {
            Some(r) => self.code.mov_reg_addr(r, addr),
            None => self.code.mov_reg_addr(tmp, addr).mov_addr_reg(dest.virt_loc, tmp),
        };
        self
    }

    pub fn mov_index(&self, dest: VReg, base: VReg, index: VReg, offset: i32) -> &Self {
        let tmp = self.mov_to_temp(base);
        let ind = self.get(index);
        match ind.host_reg {
            Some(r) => self.code.add_reg_reg(tmp, r),
            None => self.code.add_reg_addr(tmp, ind.virt_loc),
        };
        let dest = self.get(dest);
        let addr = Address::sib(1, tmp, VMEM_ADDR_REG, offset);
        match dest.host_reg {
            Some(r) => self.code.mov_reg_addr(r, addr),
            None => self.code.mov_reg_addr(tmp, addr).mov_addr_reg(dest.virt_loc, tmp),
        };
        self
    }

    pub fn mov_imm16(&self, dest: VReg, imm: i16) -> &Self {
        let reg = self.get(dest);
        match reg.host_reg {
            Some(r) => self.code.mov_reg_imm(r, imm as i64),
            None => self.code.mov_addr_imm32(reg.virt_loc, imm as i32),
        };
        self
    }

    // pub fn add_reg(&self, src: &VReg) -> &Self {
    //     let src = src.mov_spill_to_temp(e);
    //     match reg.host_reg {
    //         Some(r) => e.add_reg_reg(r, src),
    //         None => e.add_addr_reg(reg.virt_loc, src),
    //     };
    //     self
    // }
}

#[cfg(test)]
mod tests {}
