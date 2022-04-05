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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Operand {
    Reg(VReg),
    Ptr(VReg),
    PtrOffset(VReg, i32),
    Imm(i32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Opcode {
    MOVi(VReg, i16),
    MOVr(VReg, VReg),
    PUSH(VReg),
    POP(VReg),
}

pub struct Instr {
    pub opcode: Opcode,
    // TODO delete
    pub operands: [Option<Operand>; 3],
    cond: Cond,
}
