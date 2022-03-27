#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum VReg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
}

pub enum Operand {
    Reg(VReg),
    // Will probably include scale and offsets later
    Ptr(VReg),
    // Don't know if this will work yet
    // Imm(u32),
}

pub enum IR {
    MOV(Operand, Operand),
}
