pub mod instruction;
use super::{bit, bits};
use crate::ir::{Address::*, Offset::*, Opcode, Opcode::*, VReg, VReg::*};

/// Number of lookahead bytes in ARM mode
const PC_LA_THUMB: u32 = 4;
/// Number of lookahead bytes in THUMB mode
const PC_LA_ARM: u32 = 8;

/// Align an address to 32-bit boundary by zeroing out the lowest two bits
fn word_align(addr: u32) -> u32 {
    addr & !(0b11)
}

// Branch with PC-relative offset
pub fn branch_arm(instr: u32) -> Opcode {
    // Should be signed - is this right?
    let target = Relative(PC, Immediate(bits(instr, 0..23) << 4));
    match bit(instr, 24) {
        false => B(target),
        true => BL(target),
    }
}

pub fn ldr_imm_thumb(instr: u32) -> Opcode {
    // Offset is in number of words, i.e. a multiple of 4
    let offset = bits(instr, 6..10) << 2;
    let reg = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    LDR(reg, Relative(base, Immediate(offset)), None)
}

pub fn ldr_imm_sp_thumb(instr: u32) -> Opcode {
    let reg = VReg::try_from(bits(instr, 8..10)).unwrap();
    // Offset is in number of words, i.e. a multiple of 4
    let offset = bits(instr, 0..7) << 2;
    LDR(reg, Relative(SP, Immediate(offset)), None) // signed?
}

pub fn ldr_imm_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldr_literal_thumb(addr: u32, instr: u32) -> Opcode {
    // LDR (literal)
    let reg = VReg::try_from(bits(instr, 8..10)).unwrap();
    // Offset is in number of words, i.e. a multiple of 4
    let offset = bits(instr, 0..7) << 2;
    // Encode PC-relative offsets as an absolute address, since emitted code won't have access
    // to emulated PC
    let pc = addr + PC_LA_THUMB;
    return LDR(reg, Absolute(word_align(pc) + offset), None);
}

pub fn ldr_literal_arm(addr: u32, instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldr_reg_thumb(instr: u32) -> Opcode {
    let dest = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    let index = VReg::try_from(bits(instr, 6..8)).unwrap();
    LDR(dest, Relative(base, Index(index)), None)
}

pub fn ldr_reg_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrb_imm_thumb(instr: u32) -> Opcode {
    let offset = bits(instr, 6..10);
    let reg = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    LDRB(reg, Relative(base, Immediate(offset)), None)
}

pub fn ldrb_imm_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrb_literal(addr: u32, instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrb_reg_thumb(instr: u32) -> Opcode {
    let dest = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    let index = VReg::try_from(bits(instr, 6..8)).unwrap();
    LDR(dest, Relative(base, Index(index)), None)
}

pub fn ldrb_reg_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrbt(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrd_imm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrd_literal(addr: u32, instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrd_reg(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrex(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrexb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrexd(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrexh(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrh_imm_thumb(instr: u32) -> Opcode {
    let dest = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    let offset = bits(instr, 6..10) << 1;
    LDRH(dest, Relative(base, Immediate(offset)), None)
}

pub fn ldrh_imm_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrh_literal(addr: u32, instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrh_reg_thumb(instr: u32) -> Opcode {
    let dest = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    let index = VReg::try_from(bits(instr, 6..8)).unwrap();
    LDRH(dest, Relative(base, Index(index)), None)
}

pub fn ldrh_reg_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrht(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsb_imm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsb_literal(addr: u32, instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsb_reg_thumb(instr: u32) -> Opcode {
    let dest = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    let index = VReg::try_from(bits(instr, 6..8)).unwrap();
    LDRSB(dest, Relative(base, Index(index)), None)
}

pub fn ldrsb_reg_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsbt(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsh_imm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsh_literal(addr: u32, instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsh_reg_thumb(instr: u32) -> Opcode {
    let dest = VReg::try_from(bits(instr, 0..2)).unwrap();
    let base = VReg::try_from(bits(instr, 3..5)).unwrap();
    let index = VReg::try_from(bits(instr, 6..8)).unwrap();
    LDRSH(dest, Relative(base, Index(index)), None)
}

pub fn ldrsh_reg_arm(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrsht(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn ldrt(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn str_imm_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn str_imm_sp_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn str_reg_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn strb_imm_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn strb_reg_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn strh_imm_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

pub fn strh_reg_thumb(instr: u32) -> Opcode {
    panic!("Unimplemented instruction")
}

#[cfg(test)]
mod tests {
    use super::word_align;

    #[test]
    fn test_word_align() {
        assert_eq!(word_align(0x84abfe43), 0x84abfe40);
        assert_eq!(word_align(0x7f), 0x7c);
        assert_eq!(word_align(0x43a), 0x438);
    }
}
