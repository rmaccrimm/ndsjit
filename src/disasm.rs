use super::ir::{Address::*, Offset::*, Opcode, Opcode::*, VReg, VReg::*, WriteBack::*};
use std::ops::Range;

/// Number of lookahead bytes in ARM mode
const PC_LA_THUMB: u32 = 4;
/// Number of lookahead bytes in THUMB mode
const PC_LA_ARM: u32 = 8;

fn bits(word: u32, r: Range<usize>) -> u32 {
    assert!(r.start < r.end);
    (word >> r.start) & ((1 << (r.end - r.start + 1)) - 1)
}

fn bit(word: u32, b: usize) -> u32 {
    (word >> b) & 1
}

/// Align an address to 32-bit boundary by zeroing out the lowest two bits
fn word_align(addr: u32) -> u32 {
    addr & !(0b11)
}

pub fn try_disasm_arm(addr: u32, instr_bytes: &[u8; 4]) -> Option<Opcode> {
    let instr = u32::from_le_bytes(*instr_bytes);
    None
}

pub fn try_disasm_thumb(addr: u32, instr_bytes: &[u8; 2]) -> Option<Opcode> {
    let instr = u16::from_le_bytes(*instr_bytes) as u32;
    match bits(instr, 13..15) {
        0b010 => match bits(instr, 11..12) {
            0b00 => None, // HiReg/BX
            _ => Some(disasm_ldr_str_thumb(addr, instr)),
        },
        0b011 | 0b100 => Some(disasm_ldr_str_thumb(addr, instr)),
        _ => None,
    }
}

fn disasm_ldr_str_arm(addr: u32, instr: u32) -> Opcode {
    panic!()
}

fn disasm_ldr_str_thumb(addr: u32, instr: u32) -> Opcode {
    if bits(instr, 13..15) == 0b011 {
        // LDR (immediate, Thumb)
        let offset = Some(Immediate(bits(instr, 6..10) as i16));
        let reg = VReg::try_from(bits(instr, 0..2)).unwrap();
        let base = VReg::try_from(bits(instr, 3..5)).unwrap();
        return match bits(instr, 11..12) {
            0b00 => STR(Relative(base, offset), reg, None),
            0b01 => LDR(reg, Relative(base, offset), None),
            _ => panic!(),
        };
    } else if bits(instr, 11..15) == 0b01001 {
        // LDR (literal)
        let reg = VReg::try_from(bits(instr, 8..10)).unwrap();
        // Offset is in number of words, i.e. a multiple of 4
        let offset = bits(instr, 0..7) << 2;
        // Encode PC-relative offsets as an absolute address, since emitted code won't have access
        // to emulated PC
        let pc = addr + PC_LA_THUMB;
        return LDR(reg, Absolute(word_align(pc) + offset), None);
    }
    panic!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits() {
        let bytes: [u8; 4] = [0b00000000, 0b11111110, 0b00110010, 0b01110111];
        assert_eq!(
            bits(u32::from_le_bytes(bytes), 3..27),
            0b0111001100101111111000000
        );
        assert_eq!(bits(u32::from_le_bytes(bytes), 19..24), 0b100110);
        assert_eq!(bits(0b10111100, 5..12), 0b00000101);
    }

    #[test]
    fn test_word_align() {
        assert_eq!(word_align(0x84abfe43), 0x84abfe40);
        assert_eq!(word_align(0x7f), 0x7c);
        assert_eq!(word_align(0x43a), 0x438);
    }

    #[test]
    fn test_disasm_ldr_literal() {
        // 08028d6e 06 4a           ldr        r2,[DAT_08028d88]
        assert_eq!(
            try_disasm_thumb(0x08028d6e, &[0x06, 0x4a]).unwrap(),
            LDR(R2, Absolute(0x08028d88), None)
        );
        // 08028d78 04 49           ldr        r1,[DAT_08028d8c]
        assert_eq!(
            try_disasm_thumb(0x08028d78, &[0x04, 0x49]).unwrap(),
            LDR(R1, Absolute(0x08028d8c), None)
        );
    }
}
