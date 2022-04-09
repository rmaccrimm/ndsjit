#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

enum Cond {
    None,
}

#[derive(Copy, Clone, Debug)]
pub enum IntType {
    Byte,
    HalfWord,
    Word,
    DoubleWord,
}

#[derive(Copy, Clone, Debug)]
pub enum Offset {
    Immediate(u16),
    Index(VReg),
    ShiftIndex(VReg, u16),
}

#[derive(Copy, Clone, Debug)]
pub enum WriteBack {
    None,
    Pre,
    Post,
}

#[derive(Copy, Clone, Debug)]
pub struct Address {
    base: VReg,
    offset: Option<Offset>,
}

#[derive(Copy, Clone, Debug)]
pub enum Opcode {
    MOVi(VReg, i16),
    MOVr(VReg, VReg),
    PUSH(VReg),
    POP(VReg),
    STR(VReg, Address, WriteBack),
}
use Opcode::*;

pub trait RegOps {
    fn vreg_ops(self) -> (Option<VReg>, Option<VReg>, Option<VReg>);
}

impl RegOps for Opcode {
    /// Get registers operands (if present) of an instruction
    fn vreg_ops(self) -> (Option<VReg>, Option<VReg>, Option<VReg>) {
        match self {
            MOVi(r, _) => (Some(r), None, None),
            MOVr(rd, rs) => (Some(rd), Some(rs), None),
            PUSH(r) => (Some(r), None, None),
            POP(r) => (Some(r), None, None),
            _ => (None, None, None),
        }
    }
}

pub struct Instr {
    pub opcode: Opcode,
    cond: Cond,
}
