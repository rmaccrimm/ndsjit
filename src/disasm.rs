use super::ir::{Opcode, Opcode::*};
use std::ops::Range;

fn le_to_word(bytes: &[u8]) -> u32 {
    assert!(bytes.len() <= 4);
    bytes
        .iter()
        .enumerate()
        .fold(0, |acc, (i, &b)| acc | ((b as u32) << (8 * i)))
}

fn bits(bytes: &[u8], r: Range<usize>) -> u32 {
    assert!(r.start < r.end);
    (le_to_word(bytes) >> r.start) & ((1 << (r.end - r.start + 1)) - 1)
}

fn bit(bytes: &[u8], b: usize) -> u32 {
    (le_to_word(bytes) >> b) & 1
}

pub fn try_disasm_arm(instr: &[u8; 4]) -> Option<Opcode> {
    None
}

pub fn try_disasm_thumb(instr: &[u8; 2]) -> Option<Opcode> {
    match (bits(instr, 12..15), bit(instr, 9)) {
        (0b0101, 0) => Some(disasm_ldr_str_thumb(instr)),
        _ => None,
    }
}

fn disasm_ldr_str_arm(instr: &[u8; 4]) -> Opcode {}

fn disasm_ldr_str_thumb(instr: &[u8; 2]) -> Opcode {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_word() {
        assert_eq!(le_to_word(&[0xe3, 0xf1, 0x18, 0x77]), 0x7718f1e3);
        assert_eq!(le_to_word(&[0x91, 0xab, 0x01]), 0x0001ab91);
        assert_eq!(le_to_word(&[]), 0);
    }

    #[test]
    fn test_bits() {
        assert_eq!(
            bits(&[0b00000000, 0b11111110, 0b00110010, 0b01110111], 3..27),
            0b0111001100101111111000000
        );
        assert_eq!(
            bits(&[0b10001100, 0b01000110, 0b10100000, 0b10110001], 19..24),
            0b110100
        );
        assert_eq!(bits(&[0b10111100], 5..12), 0b00000101);
    }
}
