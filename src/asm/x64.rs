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
fn rex_prefix(w: bool, reg: u8, rm_or_base: u8, index: u8) -> u8 {
    let rex = if w { 0x48 } else { 0x40 };
    rex | ((reg >> 3) & 1) << 2 | ((index) >> 3 & 1) << 1 | ((rm_or_base as u8 >> 3) & 1)
}

fn mod_rm_byte(mode: u8, reg_or_op: u8, rm: u8) -> u8 {
    assert!(mode < 4);
    (mode & 0x3) << 6 | (reg_or_op & 0x7) << 3 | (rm & 0x7)
}

fn sib_byte(scale: u8, index: u8, base: u8) -> u8 {
    assert!(scale == 1 || scale == 2 || scale == 4 || scale == 8);
    let scale = [0, 0, 1, 0, 2, 0, 0, 0, 3][scale as usize] as u8;
    scale << 6 | (index & 0x7) << 3 | (base & 0x7)
}

pub fn add_rax_imm32(code: &mut Vec<u8>, imm: u32) {
    code.push(0x48);
    code.push(0x05);
    code.push((imm & 0xff) as u8);
    code.push(((imm >> 8) & 0xff) as u8);
    code.push(((imm >> 16) & 0xff) as u8);
    code.push(((imm >> 24) & 0xff) as u8);
}

pub fn mov_reg64_reg64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    code.push(rex_prefix(true, src as u8, dest as u8, 0));
    code.push(0x89);
    code.push(mod_rm_byte(3, src as u8, dest as u8));
}

pub fn mov_reg64_ptr64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    // RBP is a special case, and can't be used as address without offset, since mod=b0, rm=b101 is
    // reserved for disp32-only mode (rip relative)
    if src == RegX64::RBP || src == RegX64::R13 {
        return mov_reg64_ptr64_disp8(code, dest, src, 0);
    }
    code.push(rex_prefix(true, dest as u8, src as u8, 0));
    code.push(0x8b);
    code.push(mod_rm_byte(0, dest as u8, src as u8));
    // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
    // base is the same as modr/m (b100) -> 00100100
    if src == RegX64::RSP || src == RegX64::R12 {
        code.push(sib_byte(1, 4, src as u8))
    }
}

pub fn mov_ptr64_reg64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    if dest == RegX64::RBP || dest == RegX64::R13 {
        return mov_ptr64_reg64_disp8(code, dest, src, 0);
    }
    code.push(rex_prefix(true, src as u8, dest as u8, 0));
    code.push(0x89);
    code.push(mod_rm_byte(0, src as u8, dest as u8));
    // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
    // base is the same as modr/m (b100) -> 00100100
    if dest == RegX64::RSP || dest == RegX64::R12 {
        code.push(0x24)
    }
}

pub fn mov_reg64_ptr64_disp8(code: &mut Vec<u8>, dest: RegX64, src: RegX64, disp: i8) {
    code.push(rex_prefix(true, dest as u8, src as u8, 0));
    code.push(0x8b);
    code.push(mod_rm_byte(1, dest as u8, src as u8));
    if src == RegX64::RSP || src == RegX64::R12 {
        code.push(sib_byte(1, 4, src as u8))
    }
    code.push(disp as u8);
}

pub fn mov_ptr64_reg64_disp8(code: &mut Vec<u8>, dest: RegX64, src: RegX64, disp: i8) {
    code.push(rex_prefix(true, src as u8, dest as u8, 0));
    code.push(0x89);
    code.push(mod_rm_byte(1, src as u8, dest as u8));
    if dest == RegX64::RSP || dest == RegX64::R12 {
        code.push(sib_byte(1, 4, dest as u8))
    }
    code.push(disp as u8);
}

pub fn mov_reg64_ptr64_sib(
    code: &mut Vec<u8>,
    dest: RegX64,
    base: RegX64,
    index: RegX64,
    scale: u8,
) {
    assert!(index != RegX64::RSP);
    let mode = (base == RegX64::RBP || base == RegX64::R13) as u8;
    code.push(rex_prefix(true, dest as u8, base as u8, index as u8));
    code.push(0x8b);
    code.push(mod_rm_byte(mode, dest as u8, 0x4));
    code.push(sib_byte(scale, index as u8, base as u8));
    if mode == 1 {
        code.push(0);
    }
}

pub fn push_reg64(code: &mut Vec<u8>, reg: RegX64) {
    if (reg as u8) >= 8 {
        // For extended 64-bit registers (R8-15), reg msb is stored in the REX prefix
        code.push(rex_prefix(false, 0, reg as u8, 0));
    }
    code.push(0x50 | (reg as u8 & 0x7))
}

pub fn pop_reg64(code: &mut Vec<u8>, reg: RegX64) {
    if (reg as u8) >= 8 {
        code.push(rex_prefix(false, 0, reg as u8, 0));
    }
    code.push(0x58 | (reg as u8 & 0x7));
}

pub fn push_ptr64(code: &mut Vec<u8>, reg: RegX64) {
    let mode = if reg == RegX64::RBP || reg == RegX64::R13 {
        1
    } else {
        0
    };
    if (reg as u8) >= 8 {
        // For extended 64-bit registers (R8-15), reg msb is stored in the REX prefix
        code.push(rex_prefix(false, 0, reg as u8, 0));
    }
    code.push(0xff | (reg as u8 & 0x7));
    code.push(mod_rm_byte(mode, 0x6, reg as u8));
    match reg {
        // Set a disp8 of 0
        RegX64::RBP | RegX64::R13 => code.push(0),
        // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
        // base is the same as modr/m (b100) -> 00100100
        RegX64::RSP | RegX64::R12 => code.push(0x24),
        _ => (),
    }
}

pub fn pop_ptr64(code: &mut Vec<u8>, reg: RegX64) {
    let mode = if reg == RegX64::RBP || reg == RegX64::R13 {
        1
    } else {
        0
    };
    if (reg as u8) >= 8 {
        // For extended 64-bit registers (R8-15), reg msb is stored in the REX prefix
        code.push(rex_prefix(false, 0, reg as u8, 0));
    }
    code.push(0x8f | (reg as u8 & 0x7));
    code.push(mod_rm_byte(mode, 0, reg as u8));
    match reg {
        // Set a disp8 of 0
        RegX64::RBP | RegX64::R13 => code.push(0),
        // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
        // base is the same as modr/m (b100) -> 00100100
        RegX64::RSP | RegX64::R12 => code.push(0x24),
        _ => (),
    }
}

pub fn ret(code: &mut Vec<u8>) {
    code.push(0xc3)
}

#[cfg(test)]
mod tests;
