use super::*;
use crate::ir::{Address::*, Offset::*, Opcode::*, VReg::*};

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
fn test_disasm_ldr_imm_thumb() {
    // 08028dc8 29 68           ldr        r1,[r5,#0x0]
    assert_eq!(
        try_disasm_thumb(0x08028dc8, &[0x29, 0x68]).unwrap(),
        LDR(R1, Relative(R5, Immediate(0)), None)
    );
    // 00000000 e9 6d           ldr        r1,[r5,#0x5c]
    assert_eq!(
        try_disasm_thumb(0x0, &[0xe9, 0x6d]).unwrap(),
        LDR(R1, Relative(R5, Immediate(92)), None)
    );
}

#[test]
fn test_disasm_ldr_imm_sp_thumb() {
    // 0802c934 02 9a           ldr        r2,[sp,#local_3c]
    // -> sp  at 0x44 locally, so 3c two words above
    assert_eq!(
        try_disasm_thumb(0x0802c934, &[0x02, 0x9a]).unwrap(),
        LDR(R2, Relative(SP, Immediate(8)), None)
    );
}

#[test]
fn test_disasm_ldr_literal_thumb() {
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

#[test]
fn test_disasm_ldr_reg_thumb() {
    assert_eq!(
        try_disasm_thumb(0, &[0xdc, 0x59]).unwrap(),
        LDR(R4, Relative(R3, Index(R7)), None)
    )
}

#[test]
fn test_disasm_ldrb_imm_thumb() {
    // 0802df76 a5 7e           ldrb       r5,[r4,#0x1a]
    assert_eq!(
        try_disasm_thumb(0x0802df76, &[0xa5, 0x7e]).unwrap(),
        LDRB(R5, Relative(R4, Immediate(0x1a)), None)
    );
}

#[test]
fn test_disasm_ldrb_reg_thumb() {
    assert_eq!(
        try_disasm_thumb(0, &[0x48, 0x5d]).unwrap(),
        LDR(R0, Relative(R1, Index(R5)), None)
    )
}

#[test]
fn test_disasm_ldrh_imm_thumb() {
    // 080e22ae 68 88           ldrh       r0,[r5,#0x2]
    assert_eq!(
        try_disasm_thumb(0, &[0x68, 0x88]).unwrap(),
        LDRH(R0, Relative(R5, Immediate(2)), None)
    );
    // 08028f68 01 8f           ldrh       r1,[r0,#0x38]
    assert_eq!(
        try_disasm_thumb(0, &[0x01, 0x8f]).unwrap(),
        LDRH(R1, Relative(R0, Immediate(0x38)), None)
    );
}

#[test]
fn ldrh_reg_thumb() {
    assert_eq!(
        try_disasm_thumb(0, &[0x17, 0x5b]).unwrap(),
        LDRH(R7, Relative(R2, Index(R4)), None)
    );
}

#[test]
fn ldrsb_reg_thumb() {
    assert_eq!(
        try_disasm_thumb(0, &[0xc8, 0x56]).unwrap(),
        LDRSB(R0, Relative(R1, Index(R3)), None)
    );
}

#[test]
fn ldrsh_reg_thumb() {
    assert_eq!(
        try_disasm_thumb(0, &[0x6e, 0x5e]).unwrap(),
        LDRSH(R6, Relative(R5, Index(R1)), None)
    );
}

#[test]
fn test_disasm_str_imm_thumb() {
    // 08028e92 08 87           strh       r0,[r1,#0x38]
}
