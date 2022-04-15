use std::convert::TryFrom;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VReg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    SP = 13,
    LR = 14,
    PC = 15,
}
use VReg::*;

impl TryFrom<u32> for VReg {
    type Error = ();
    fn try_from(r: u32) -> Result<Self, Self::Error> {
        let regs = [
            R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, SP, LR, PC,
        ];
        if r > 15 {
            Err(())
        } else {
            Ok(regs[r as usize])
        }
    }
}

enum Cond {
    None,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Offset {
    Immediate(u32), // size?
    Index(VReg),
    ShiftIndex(VReg, u16),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WriteBack {
    Pre,
    Post,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Relative(VReg, Offset),
    Absolute(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    // MOVi(VReg, i16),
    // MOVr(VReg, VReg),
    // PUSH(VReg),
    // POP(VReg),
    // STR(Address, VReg, Option<WriteBack>),
    LDR(VReg, Address, Option<WriteBack>),
    // STRB(Address, VReg, Option<WriteBack>),
    LDRB(VReg, Address, Option<WriteBack>),
    // STRH(Address, VReg, Option<WriteBack>),
    LDRH(VReg, Address, Option<WriteBack>),
    LDRSB(VReg, Address, Option<WriteBack>),
    LDRSH(VReg, Address, Option<WriteBack>),
}
// use Opcode::*;

// pub trait RegOps {
//     fn vreg_ops(self) -> (Option<VReg>, Option<VReg>, Option<VReg>);
// }

// impl RegOps for Opcode {
//     /// Get registers operands (if present) of an instruction
//     fn vreg_ops(self) -> (Option<VReg>, Option<VReg>, Option<VReg>) {
//         match self {
//             MOVi(r, _) => (Some(r), None, None),
//             MOVr(rd, rs) => (Some(rd), Some(rs), None),
//             PUSH(r) => (Some(r), None, None),
//             POP(r) => (Some(r), None, None),
//             _ => (None, None, None),
//         }
//     }
// }

pub struct Instr {
    pub opcode: Opcode,
    cond: Cond,
}
