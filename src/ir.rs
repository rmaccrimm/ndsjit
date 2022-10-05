use crate::{asm::AssemblerX64, jit::JITCompiler};
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

#[derive(Copy, Clone, Debug)]
enum Cond {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    None,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WriteBack {
    Pre,
    Post,
}

pub trait CodeGen {
    type Compiler;
    
    fn must_interpret(&self) -> bool;
    fn codegen(&self, jit: Self::Compiler);
}

pub enum ImmValue {
    Word(u32),
    HalfWord(u16),
    Byte(u8),
}

pub enum Operand {
    VReg(VReg),
    Imm(ImmValue),
}

pub struct BinaryInstr {
    cond: Cond,
    op1: Operand,
    op2: Operand,
    
}

#[derive(Copy, Clone, Debug)]
pub struct LDR {
    dest: VReg,
    src: Address,
    writeback: Option<WriteBack>,
}

impl CodeGen for LDR {
    fn must_interpret(&self) -> bool {
        false
    }

    fn codegen(&self, asm: &mut AssemblerX64) {
        match self.src {
            Address::Absolute(addr) => asm.ldr_abs(self.dest, addr),
            Address::Relative(base, off) => match off {
                Offset::Immediate(imm) => asm.ldr_rel(self.dest, base, imm),
                Offset::Index(ind) => asm.ldr_rel_ind(self.dest, base, ind, 0),
                Offset::ShiftIndex(ind, shift) => {
		    let tmp = asm.create_temp();
		    asm.mov_reg(tmp, ind);
		    asm.shift(tmp, shift);
		    asm.ldr_rel_ind(self.dest, base, tmp, 0);
		}
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LDRB {
    cond: Cond,
    dest: VReg,
    src: Address,
    writeback: Option<WriteBack>,
}

#[derive(Copy, Clone, Debug)]
pub struct LDRH {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct LDRSB {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct LDRSH {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct STRB {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct STRH {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct B {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct BX {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct BL {
    cond: Cond,
}

#[derive(Copy, Clone, Debug)]
pub struct NOP {
    cond: Cond,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    // MOVi(VReg, i16),
    // MOVr(VReg, VReg),
    // PUSH(VReg),
    // POP(VReg),
    // STR(Address, VReg, Option<WriteBack>),
    LDR(LDR),
    // STRB(Address, VReg, Option<WriteBack>),
    LDRB(LDRB),
    // STRH(Address, VReg, Option<WriteBack>),
    LDRH(LDRH),
    LDRSB(LDRSB),
    LDRSH(LDRSH),
    B(B),
    BX(BX),
    BL(BL),
    NOP,
}

impl Opcode {
    pub fn requires_interpreter(&self) -> bool {
        match self {
            Opcode::B(addr) | Opcode::BX(addr) | Opcode::BL(addr) => match addr {
                Address::Absolute(_) => false,
                Address::Relative(_, _) => true,
            },
            _ => false,
        }
    }
}

pub struct Instr {
    pub opcode: Opcode,
    cond: Cond,
}
