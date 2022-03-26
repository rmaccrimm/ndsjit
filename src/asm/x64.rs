use std::vec::Vec;

#[derive(Copy, Clone)]
pub enum RegX64 {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

fn write_bytes(code: &mut Vec<u8>, bytes: &[u8]) {
    for &b in bytes {
        code.push(b);
    }
}

// Bits 3-7 are constant, bit 2 is the msb of the register field, bit 0 is the msb of the r/m field
fn rex_prefix(reg: RegX64, rm: RegX64) -> u8 {
    0x48 | ((reg as u8 >> 3) & 1) << 2 | ((rm as u8 >> 3) & 1)
}

fn mod_rm_byte(mod_: u8, reg: RegX64, rm: RegX64) -> u8 {
    assert!(mod_ < 4);
    (mod_ & 0x3) << 6 | (reg as u8 & 0x7) << 3 | (rm as u8 & 0x7)
}

pub fn add_rax_imm32(code: &mut Vec<u8>, imm: u32) {
    write_bytes(
        code,
        &[
            0x48,
            0x05,
            (imm & 0xff) as u8,
            ((imm >> 8) & 0xff) as u8,
            ((imm >> 16) & 0xff) as u8,
            ((imm >> 24) & 0xff) as u8,
        ],
    )
}

pub fn mov_reg64_reg64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    write_bytes(
        code,
        &[rex_prefix(src, dest), 0x8b, mod_rm_byte(3, src, dest)],
    )
}

pub fn mov_reg64_ptr64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    write_bytes(
        code,
        &[rex_prefix(dest, src), 0x8b, mod_rm_byte(0, dest, src)],
    )
}

pub fn mov_ptr64_reg64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    write_bytes(
        code,
        &[rex_prefix(src, dest), 0x89, mod_rm_byte(0, src, dest)],
    )
}

pub fn ret(code: &mut Vec<u8>) {
    code.push(0xc3)
}
