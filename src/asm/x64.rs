use std::vec::Vec;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
fn rex_prefix(w: bool, reg: u8, rm: u8) -> u8 {
    let rex = if w { 0x48 } else { 0x40 };
    rex | ((reg as u8 >> 3) & 1) << 2 | ((rm as u8 >> 3) & 1)
}

fn mod_rm_byte(mod_: u8, reg_or_op: u8, rm: u8) -> u8 {
    assert!(mod_ < 4);
    (mod_ & 0x3) << 6 | (reg_or_op & 0x7) << 3 | (rm & 0x7)
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
        &[
            rex_prefix(true, src as u8, dest as u8),
            0x89,
            mod_rm_byte(3, src as u8, dest as u8),
        ],
    )
}

pub fn mov_reg64_ptr64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    // RBP is a special case, and can't be used as address without offset, since mod=b0, rm=b101 is
    // reserved for disp32-only mode (rip relative)
    let mod_ = if src == RegX64::RBP || src == RegX64::R13 {
        1
    } else {
        0
    };
    write_bytes(
        code,
        &[
            rex_prefix(true, dest as u8, src as u8),
            0x8b,
            mod_rm_byte(mod_, dest as u8, src as u8),
        ],
    );
    match src {
        // Set a disp8 of 0
        RegX64::RBP | RegX64::R13 => code.push(0),
        // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
        // base is the same as modr/m (b100) -> 00100100
        RegX64::RSP | RegX64::R12 => code.push(0x24),
        _ => (),
    }
}

pub fn mov_ptr64_reg64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    // RBP is a special case, and can't be used as address without offset, since mod=b0, rm=b101 is
    // reserved for disp32-only mode (rip relative)
    let mod_ = (dest == RegX64::RBP || dest == RegX64::R13) as u8;
    write_bytes(
        code,
        &[
            rex_prefix(true, src as u8, dest as u8),
            0x89,
            mod_rm_byte(mod_, src as u8, dest as u8),
        ],
    );
    match dest {
        // Set a disp8 of 0
        RegX64::RBP | RegX64::R13 => code.push(0),
        // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
        // base is the same as modr/m (b100) -> 00100100
        RegX64::RSP | RegX64::R12 => code.push(0x24),
        _ => (),
    }
}

pub fn push_reg64(code: &mut Vec<u8>, reg: RegX64) {
    if (reg as u8) >= 8 {
        // For extended 64-bit registers (R8-15), reg msb is stored in the REX prefix
        code.push(rex_prefix(false, 0, reg as u8));
    }
    code.push(0x50 | (reg as u8 & 0x7))
}

pub fn push_ptr64(code: &mut Vec<u8>, reg: RegX64) {
    let mod_ = if reg == RegX64::RBP || reg == RegX64::R13 {
        1
    } else {
        0
    };
    if (reg as u8) >= 8 {
        // For extended 64-bit registers (R8-15), reg msb is stored in the REX prefix
        code.push(rex_prefix(false, 0, reg as u8));
    }
    write_bytes(
        code,
        &[0xff | (reg as u8 & 0x7), mod_rm_byte(mod_, 0x6, reg as u8)],
    );
    match reg {
        // Set a disp8 of 0
        RegX64::RBP | RegX64::R13 => code.push(0),
        // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
        // base is the same as modr/m (b100) -> 00100100
        RegX64::RSP | RegX64::R12 => code.push(0x24),
        _ => (),
    }
}

pub fn pop_reg64(code: &mut Vec<u8>, reg: RegX64) {}

pub fn pop_ptr64(code: &mut Vec<u8>, reg: RegX64) {}

pub fn ret(code: &mut Vec<u8>) {
    code.push(0xc3)
}

#[cfg(test)]
mod tests;
