use super::{Instr, Operand, RegX64, VReg};
use crate::ir::Opcode;

use std::vec::Vec;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MappedReg {
    Phys(RegX64),
    Spill(u8),
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
                MappedReg::Phys(RegX64::RAX),
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
                MappedReg::Spill(5),
                MappedReg::Spill(6),
                MappedReg::Spill(7),
                MappedReg::Spill(8),
                MappedReg::Spill(9),
                MappedReg::Spill(10),
                MappedReg::Spill(11),
                MappedReg::Spill(12),
                MappedReg::Spill(13),
                MappedReg::Spill(14),
                MappedReg::Spill(15),
                MappedReg::Spill(16),
                MappedReg::Spill(17),
            ],
            num_spilled: 18,
        }
    }

    // Just uses a naive allocation right now, with registers mapped to physical registers in the
    // order they appear
    pub fn new(instructions: Vec<Instr>) -> RegAllocation {
        // Scan through instructions to get a list of registers used
        // Use a vec atm to keep regs in order, but probably slow
        let mut vregs: Vec<VReg> = Vec::new();
        for ir in instructions {
            for op in ir.operands {
                if let Some(Operand::Reg(reg)) | Some(Operand::Ptr(reg)) = op {
                    if !vregs.contains(&reg) {
                        vregs.push(reg);
                    }
                }
            }
        }
        // Currently RBP and RSP are reserved (should have a better way of doing this)
        let available_pregs = [
            RegX64::RAX,
            RegX64::RCX, // TODO reserve RDX and RCX
            RegX64::RDX,
            RegX64::RBX,
            RegX64::RSI,
            RegX64::RDI,
            RegX64::R8,
            RegX64::R9,
            RegX64::R10,
            RegX64::R11,
            RegX64::R12,
            RegX64::R13,
            RegX64::R14,
            RegX64::R15,
        ];
        let mut mapping = vec![MappedReg::Unmapped; 30];
        let mut preg_ind = 0;
        let mut num_spilled = 0;
        for (i, vreg) in vregs.into_iter().enumerate() {
            let m = if preg_ind < available_pregs.len() {
                MappedReg::Phys(available_pregs[preg_ind])
            } else {
                num_spilled += 1;
                MappedReg::Spill((i - available_pregs.len()) as u8)
            };
            mapping[vreg as usize] = m;
            preg_ind += 1;
        }
        RegAllocation {
            mapping,
            num_spilled,
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
mod tests {
    use super::*;

    fn regop(reg: VReg) -> Option<Operand> {
        Some(Operand::Reg(reg))
    }

    fn ptrop(reg: VReg) -> Option<Operand> {
        Some(Operand::Ptr(reg))
    }

    #[test]
    fn test_swap() {
        let mut alloc = RegAllocation::new(vec![]);
        *alloc.get(VReg::R0) = MappedReg::Phys(RegX64::RDX);
        *alloc.get(VReg::R1) = MappedReg::Spill(0);

        alloc.swap(VReg::R0, VReg::R1);
        assert_eq!(*alloc.get(VReg::R0), MappedReg::Spill(0));
        assert_eq!(*alloc.get(VReg::R1), MappedReg::Phys(RegX64::RDX));
    }

    #[test]
    fn test_simple_allocation() {
        let instructions = vec![Instr {
            opcode: Opcode::MOV,
            operands: [regop(VReg::R6), regop(VReg::R0), None],
        }];
        let alloc = RegAllocation::new(instructions);
        let mut expected = vec![MappedReg::Unmapped; 30];
        expected[6] = MappedReg::Phys(RegX64::RAX);
        expected[0] = MappedReg::Phys(RegX64::RCX);
        assert_eq!(alloc.mapping, expected);
    }

    #[test]
    fn test_spill_allocation() {
        let instructions = vec![
            Instr {
                opcode: Opcode::MOV,
                operands: [regop(VReg::R2), ptrop(VReg::R0), ptrop(VReg::R1)],
            },
            Instr {
                opcode: Opcode::MOV,
                operands: [regop(VReg::R3), ptrop(VReg::R4), ptrop(VReg::R5)],
            },
            Instr {
                opcode: Opcode::MOV,
                operands: [regop(VReg::R8), ptrop(VReg::R7), ptrop(VReg::R6)],
            },
            Instr {
                opcode: Opcode::MOV,
                operands: [regop(VReg::R9), ptrop(VReg::R10), ptrop(VReg::R11)],
            },
            Instr {
                opcode: Opcode::MOV,
                operands: [regop(VReg::R12), ptrop(VReg::R13), ptrop(VReg::R14)],
            },
            Instr {
                opcode: Opcode::MOV,
                operands: [regop(VReg::R15), ptrop(VReg::R16), ptrop(VReg::R17)],
            },
        ];
        let alloc = RegAllocation::new(instructions);
        let expected = vec![
            MappedReg::Phys(RegX64::RCX), // 0
            MappedReg::Phys(RegX64::RDX), // 1
            MappedReg::Phys(RegX64::RAX), // 2
            MappedReg::Phys(RegX64::RBX), // 3
            MappedReg::Phys(RegX64::RSI), // 4
            MappedReg::Phys(RegX64::RDI), // 5
            MappedReg::Phys(RegX64::R10), // 6
            MappedReg::Phys(RegX64::R9),  // 7
            MappedReg::Phys(RegX64::R8),  // 8
            MappedReg::Phys(RegX64::R11), // 9
            MappedReg::Phys(RegX64::R12), // 10
            MappedReg::Phys(RegX64::R13), // 11
            MappedReg::Phys(RegX64::R14), // 12
            MappedReg::Phys(RegX64::R15), // 13
            MappedReg::Spill(0),          // 14
            MappedReg::Spill(1),          // 15
            MappedReg::Spill(2),          // 16
            MappedReg::Spill(3),          // 17
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
            MappedReg::Unmapped,
        ];
        assert_eq!(alloc.mapping, expected);
    }
}
