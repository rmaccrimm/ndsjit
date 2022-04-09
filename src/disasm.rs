use super::ir::Opcode;
use std::ops::Range;

pub fn bits(byte: u8, r: Range<usize>) -> u8 {
    assert!(r.start < r.end);
    (byte >> r.start) & ((1 << (r.end - r.start + 1)) - 1)
}

pub fn bit(byte: u8, b: usize) -> u8 {
    (byte >> b) & 1
}

pub fn try_disasm_arm(instr: &[u8; 4]) -> Option<Opcode> {
    None
}

pub fn try_disasm_thumb(instr: &[u8; 2]) -> Option<Opcode> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bits() {
        assert_eq!(bits(0b11101010, 0..3), 0b1010);
        assert_eq!(bits(0b01000110, 4..7), 0b0100);
        assert_eq!(bits(0b11101011, 5..6), 0b11);
        assert_eq!(bit(0b11101011, 7), 1);
        assert_eq!(bit(0b11101011, 2), 0);
    }
}
