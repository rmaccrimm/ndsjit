use std::ops::Range;

/// Get value of bit in positions start..end (inclusive)
pub fn bits(word: u32, r: Range<usize>) -> u32 {
    assert!(r.start < r.end);
    (word >> r.start) & ((1 << (r.end - r.start + 1)) - 1)
}

/// Get value of a single bit
pub fn bit(word: u32, b: usize) -> u32 {
    (word >> b) & 1
}

/// Align an address to 32-bit boundary by zeroing out the lowest two bits
fn word_align(addr: u32) -> u32 {
    addr & !(0b11)
}
