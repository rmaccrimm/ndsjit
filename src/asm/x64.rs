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
    // RBP is a special case, and can't be used as address without offset, since mod=b0, rm=b101 is
    // reserved for disp32-only mode (rip relative)
    let mod_ = if src == RegX64::RBP { 1 } else { 0 };
    write_bytes(
        code,
        &[rex_prefix(dest, src), 0x8b, mod_rm_byte(mod_, dest, src)],
    );
    match src {
        // Set a disp8 of 0
        RegX64::RBP => code.push(0),
        // An SIB byte follows all any mov with r/m field = b100. Index = b100 indicates no index,
        // base is the same as modr/m (b100) -> 00100100
        RegX64::RSP | RegX64::R12 => code.push(0x24),
        _ => (),
    }
}

pub fn mov_ptr64_reg64(code: &mut Vec<u8>, dest: RegX64, src: RegX64) {
    // RBP is a special case, and can't be used as address without offset, since mod=b0, rm=b101 is
    // reserved for disp32-only mode (rip relative)
    let mod_ = if dest == RegX64::RBP { 1 } else { 0 };
    write_bytes(
        code,
        &[rex_prefix(src, dest), 0x89, mod_rm_byte(mod_, src, dest)],
    );
    match dest {
        // Set a disp8 of 0
        RegX64::RBP => code.push(0),
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
mod tests {
    use super::*;

    #[test]
    fn test_mov_reg64_ptr64_1() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R8, RegX64::RBP);
        assert_eq!(code, vec![0x4C, 0x8B, 0x45, 0x00]); // mov r8 [rbp]
    }
    #[test]
    fn test_mov_reg64_ptr64_3() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R15, RegX64::RSI);
        assert_eq!(code, vec![0x4C, 0x8B, 0x3E]); // mov r15, [rsi]
    }
    #[test]
    fn test_mov_reg64_ptr64_4() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RDI, RegX64::RBX);
        assert_eq!(code, vec![0x48, 0x8B, 0x3B]); // mov rdi,[rbx]
    }
    #[test]
    fn test_mov_reg64_ptr64_5() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RAX, RegX64::RAX);
        assert_eq!(code, vec![0x48, 0x8B, 0x00]); // mov rax,[rax]
    }
    #[test]
    fn test_mov_reg64_ptr64_6() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R11, RegX64::RCX);
        assert_eq!(code, vec![0x4C, 0x8B, 0x19]); // mov r11,[rcx]
    }
    #[test]
    fn test_mov_reg64_ptr64_7() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RBP, RegX64::RSP);
        assert_eq!(code, vec![0x48, 0x8B, 0x2C, 0x24]); //mov rbp,[rsp]
    }
    #[test]
    fn test_mov_reg64_ptr64_8() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RCX, RegX64::RDI);
        assert_eq!(code, vec![0x48, 0x8B, 0x0F]); // mov rcx,[rdi]
    }
    #[test]
    fn test_mov_reg64_ptr64_9() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R9, RegX64::R12);
        assert_eq!(code, vec![0x4D, 0x8B, 0x0C, 0x24]) // mov r9,[r12]
    }

    #[test]
    fn test_mov_ptr64_reg64_1() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::RBP, RegX64::RDI);
        assert_eq!(code, vec![0x48, 0x89, 0x7D, 0x00]) // mov [rbp],rdi
    }

    #[test]
    fn test_mov_ptr64_reg64_2() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::RSP, RegX64::RAX);
        assert_eq!(code, vec![0x48, 0x89, 0x04, 0x24]) // mov [rsp],rax
    }

    #[test]
    fn test_mov_ptr64_reg64_3() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::R12, RegX64::R15);
        assert_eq!(code, vec![0x4D, 0x89, 0x3C, 0x24]) // mov [r12],r15
    }
}
