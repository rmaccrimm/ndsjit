use super::{Instr, RegX64, VReg};

use std::vec::Vec;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MappedReg {
    /// Mapped to a physical register
    Phys(RegX64),
    /// Spilled onto stack with index
    Spill(i8),
    /// Not mapped
    Unmapped,
}

pub struct RegAllocation {
    pub mapping: Vec<MappedReg>,
    pub num_spilled: u8,
}

impl RegAllocation {
    pub fn default() -> RegAllocation {
        RegAllocation {
            mapping: vec![
                MappedReg::Phys(RegX64::RBX),
                MappedReg::Phys(RegX64::RSI),
                MappedReg::Phys(RegX64::RDI),
                MappedReg::Phys(RegX64::R8),
                MappedReg::Phys(RegX64::R9),
                MappedReg::Phys(RegX64::R10),
                MappedReg::Phys(RegX64::R11),
                MappedReg::Phys(RegX64::R12),
                MappedReg::Phys(RegX64::R13),
                MappedReg::Phys(RegX64::R14),
                MappedReg::Phys(RegX64::R15),
                MappedReg::Spill(0),
                MappedReg::Spill(1),
                MappedReg::Spill(2),
                MappedReg::Spill(3),
                MappedReg::Spill(4),
            ],
            num_spilled: 5,
        }
    }

    // Just uses a naive allocation right now, with registers mapped to physical registers in the
    // order they appear
    // pub fn new(instructions: Vec<Instr>) -> RegAllocation {
    //     // Scan through instructions to get a list of registers used
    //     // Use a vec atm to keep regs in order, but probably slow
    //     let mut vregs: Vec<VReg> = Vec::new();
    //     for ir in instructions {
    //         for op in ir.operands {
    //             if let Some(Reg(reg)) /*| Some(Ptr(reg))*/ = op {
    //                 if !vregs.contains(&reg) {
    //                     vregs.push(reg);
    //                 }
    //             }
    //         }
    //     }
    //     // Currently RBP and RSP are reserved (should have a better way of doing this)
    //     let available_pregs = [
    //         RegX64::RBX,
    //         RegX64::RSI,
    //         RegX64::RDI,
    //         RegX64::R8,
    //         RegX64::R9,
    //         RegX64::R10,
    //         RegX64::R11,
    //         RegX64::R12,
    //         RegX64::R13,
    //         RegX64::R14,
    //         RegX64::R15,
    //     ];
    //     let mut mapping = vec![MappedReg::Unmapped; 30];
    //     let mut preg_ind = 0;
    //     let mut num_spilled = 0;
    //     for (i, vreg) in vregs.into_iter().enumerate() {
    //         let m = if preg_ind < available_pregs.len() {
    //             MappedReg::Phys(available_pregs[preg_ind])
    //         } else {
    //             num_spilled += 1;
    //             MappedReg::Spill((i - available_pregs.len()) as i8)
    //         };
    //         mapping[vreg as usize] = m;
    //         preg_ind += 1;
    //     }
    //     RegAllocation {
    //         mapping,
    //         num_spilled,
    //     }
    // }

    pub fn swap(&mut self, a: VReg, b: VReg) {
        let temp = self.mapping[a as usize];
        *self.get_mut(a) = self.get(b);
        *self.get_mut(b) = temp;
    }

    pub fn get(&mut self, reg: VReg) -> MappedReg {
        self.mapping[reg as usize]
    }

    pub fn get_mut(&mut self, reg: VReg) -> &mut MappedReg {
        &mut self.mapping[reg as usize]
    }
}

#[cfg(test)]
mod tests {}
