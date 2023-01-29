use std::ops::Range;

/// Get value of bits in positions start..end (inclusive)
pub fn bits(word: u32, r: Range<usize>) -> u32 {
    assert!(r.start < r.end);
    (word >> r.start) & ((1 << (r.end - r.start + 1)) - 1)
}

/// Takes the bits at the given positions and slides them to the right, in the order returned by the
/// iterator
pub fn pick_bits(word: u32, bits: &Vec<u32>) -> u32 {
    let mut x = 0;
    for (i, b) in bits.iter().enumerate() {
        x |= ((word >> b) & 1) << i;
    }
    x
}

/// Get value of a single bit
pub fn bit(word: u32, b: usize) -> u32 {
    (word >> b) & 1
}

/// Align an address to 32-bit boundary by zeroing out the lowest two bit_range
fn word_align(addr: u32) -> u32 {
    addr & !(0b11)
}

/// Perform bitwise comparison against bit string matching pattern [01x]. Positions with an "x" can
/// have any value, 0's and 1's must match exactly
pub fn bit_match(x: u32, pattern: &str) -> bool {
    let mask =
        u32::from_str_radix(pattern.replace("0", "1").replace("x", "0").as_str(), 2).unwrap();
    let masked = x & mask;
    let comp = u32::from_str_radix(pattern.replace("x", "0").as_str(), 2).unwrap();
    masked == comp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_match() {
        assert!(bit_match(0b11001, "11xx1"));
        assert!(bit_match(0b11011, "11xx1"));
        assert!(bit_match(0b11101, "11xx1"));
        assert!(bit_match(0b11111, "11xx1"));

        assert!(!bit_match(0b01001, "11xx1"));
        assert!(!bit_match(0b10011, "11xx1"));
        assert!(!bit_match(0b11100, "11xx1"));
        assert!(!bit_match(0b00110, "11xx1"));

        assert!(bit_match(0b00110000100, "0110xx010x"));
        assert!(bit_match(0b00110000101, "0110xx010x"));
        assert!(bit_match(0b00110010100, "0110xx010x"));
        assert!(bit_match(0b00110010101, "0110xx010x"));

        assert!(bit_match(0b00110100100, "0110xx010x"));
        assert!(bit_match(0b00110100101, "0110xx010x"));
        assert!(bit_match(0b00110110100, "0110xx010x"));
        assert!(bit_match(0b00110110101, "0110xx010x"));

        assert!(bit_match(0b10110000100, "0110xx010x"));
        assert!(bit_match(0b10110000101, "0110xx010x"));
        assert!(bit_match(0b10110010100, "0110xx010x"));
        assert!(bit_match(0b10110010101, "0110xx010x"));

        assert!(bit_match(0b10110100100, "0110xx010x"));
        assert!(bit_match(0b10110100101, "0110xx010x"));
        assert!(bit_match(0b10110110100, "0110xx010x"));
        assert!(bit_match(0b10110110101, "0110xx010x"));

        assert!(!bit_match(0b10010100100, "0110xx010x"));
        assert!(!bit_match(0b10110100111, "0110xx010x"));
        assert!(!bit_match(0b10111110100, "0110xx010x"));
        assert!(!bit_match(0b11110110101, "0110xx010x"));
    }

    #[test]
    fn test_pick_bits() {
        assert_eq!(pick_bits(0b10010101010111, &vec![1, 5, 4, 9]), 0b0101);
        assert_eq!(pick_bits(0b10010101010111, &vec![3, 4]), 0b10);
        assert_eq!(pick_bits(0b10010101010111, &vec![4, 3]), 0b01);
    }
}
