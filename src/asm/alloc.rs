use super::ir::VReg;

use std::vec::Vec;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MappedReg {
    /// Mapped to a physical register
    Phys(u8),
    /// Spilled onto stack with index
    Spill(usize),
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
                MappedReg::Phys(3),  // R0
                MappedReg::Phys(6),  // R1
                MappedReg::Phys(7),  // R2
                MappedReg::Phys(8),  // R3
                MappedReg::Phys(9),  // R4
                MappedReg::Phys(10), // R5
                MappedReg::Phys(11), // R6
                MappedReg::Phys(12), // R7
                MappedReg::Phys(13), // R8
                MappedReg::Phys(14), // R9
                MappedReg::Phys(15), // R10
                MappedReg::Spill(0), // R11
                MappedReg::Spill(1), // R12
                MappedReg::Spill(2), // SP
                MappedReg::Spill(3), // LR
                MappedReg::Spill(4), // PC
            ],
            num_spilled: 5,
        }
    }

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
