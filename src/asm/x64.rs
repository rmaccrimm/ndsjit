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

/// First 2 bits of ModR/M byte. Indicates what kind of displacement follows an instruction when
/// using a register as a pointer, that the register is being used as a value directly
#[derive(Copy, Clone)]
enum DispMode {
    NoDisp = 0,
    Disp8 = 1,
    Disp32 = 2,
    Value = 3,
}
use DispMode::*;

/// REX prefix byte, which extends the ModR/M and/or SIB bytes in 64 bit mode by encoding the msb
/// for operands. W indicates 64-bit operands, but is sometimes not needed (e.g. in push/pop)
fn rex_prefix(w: bool, reg: u8, rm_or_base: u8, index: u8) -> u8 {
    let rex = if w { 0x48 } else { 0x40 };
    rex | ((reg >> 3) & 1) << 2 | ((index) >> 3 & 1) << 1 | ((rm_or_base as u8 >> 3) & 1)
}

/// r/m value of b100 in the modr/m byte indicates the scale-index-bound addressing mode is used
const SIB_RM: u8 = 0b100;

/// ModR/M byte which encodes an addressing mode, and the 3 lsb of a register operand (reg) and
/// register or memory operand (r/m). Alternatively the reg field is sometimes an extension of the
/// instruction opcode
fn mod_rm_byte(mode: DispMode, reg_or_op: u8, rm: u8) -> u8 {
    (mode as u8 & 0x3) << 6 | (reg_or_op & 0x7) << 3 | (rm & 0x7)
}

/// SIB follows ModR/M byte when using scale-index-bound add
fn sib_byte(scale: u8, index: u8, base: u8) -> u8 {
    assert!(scale == 1 || scale == 2 || scale == 4 || scale == 8);
    let scale = [0, 0, 1, 0, 2, 0, 0, 0, 3][scale as usize] as u8;
    scale << 6 | (index & 0x7) << 3 | (base & 0x7)
}

/// Stores a vec of encoded x86_64 instructons
pub struct EmitterX64 {
    pub buf: Vec<u8>,
}

fn build_mov_ptr_instr(
    opcode: u8,
    reg: RegX64,
    ptr_reg: RegX64,
    disp_mode: DispMode,
    disp: u32,
) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(8);
    let mut mode = disp_mode;

    if reg as u8 > 7 || ptr_reg as u8 > 7 {
        bytes.push(rex_prefix(false, reg as u8, ptr_reg as u8, 0))
    }
    bytes.push(opcode);
    if ptr_reg == RegX64::RBP || ptr_reg == RegX64::R13 {
        if let NoDisp = disp_mode {
            mode = Disp8
        }
    }
    bytes.push(mod_rm_byte(mode, reg as u8, ptr_reg as u8));
    if ptr_reg == RegX64::RSP || ptr_reg == RegX64::R12 {
        // sib_byte(1, 4, ptr_reg as u8)
        bytes.push(0x24);
    }
    match mode {
        NoDisp | Value => (),
        Disp8 => bytes.push(disp as u8),
        Disp32 => bytes.extend_from_slice(&disp.to_le_bytes()),
    };
    bytes
}

impl EmitterX64 {
    pub fn new() -> EmitterX64 {
        EmitterX64 { buf: Vec::new() }
    }

    pub fn call_reg64(&mut self, reg: RegX64) -> &mut Self {
        self.buf
            .extend_from_slice(&[0xff, mod_rm_byte(Value, 2, reg as u8)]);
        self
    }

    pub fn mov_reg64_reg64(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.buf.push(rex_prefix(true, src as u8, dest as u8, 0));
        self.buf.push(0x89);
        self.buf.push(mod_rm_byte(Value, src as u8, dest as u8));
        self
    }

    pub fn mov_reg32_reg32(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        if src as u8 > 7 || dest as u8 > 7 {
            self.buf.push(rex_prefix(false, src as u8, dest as u8, 0))
        }
        self.buf.push(0x89);
        self.buf.push(mod_rm_byte(Value, src as u8, dest as u8));
        self
    }

    pub fn mov_reg32_ptr64(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.buf
            .extend_from_slice(&build_mov_ptr_instr(0x8b, dest, src, NoDisp, 0)[..]);
        self
    }

    pub fn mov_ptr64_reg32(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.buf
            .extend_from_slice(&build_mov_ptr_instr(0x89, src, dest, NoDisp, 0));
        self
    }

    pub fn mov_reg32_ptr64_disp8(&mut self, dest: RegX64, src: RegX64, disp: i8) -> &mut Self {
        self.buf
            .extend_from_slice(&build_mov_ptr_instr(0x8b, dest, src, Disp8, disp as u32)[..]);
        self
    }

    pub fn mov_ptr64_reg32_disp8(&mut self, dest: RegX64, src: RegX64, disp: i8) -> &mut Self {
        self.buf
            .extend_from_slice(&build_mov_ptr_instr(0x89, src, dest, Disp8, disp as u32)[..]);
        self
    }

    pub fn mov_reg32_ptr64_disp32(&mut self, dest: RegX64, src: RegX64, disp: i32) -> &mut Self {
        self.buf
            .extend_from_slice(&build_mov_ptr_instr(0x8b, dest, src, Disp32, disp as u32)[..]);
        self
    }

    pub fn mov_ptr64_reg32_disp32(&mut self, dest: RegX64, src: RegX64, disp: i32) -> &mut Self {
        self.buf
            .extend_from_slice(&build_mov_ptr_instr(0x89, src, dest, Disp32, disp as u32)[..]);
        self
    }

    pub fn mov_reg64_imm32(&mut self, dest: RegX64, imm: i32) -> &mut Self {
        self.buf.extend_from_slice(&[
            rex_prefix(true, 0, dest as u8, 0),
            0xc7,
            mod_rm_byte(Value, 0, dest as u8),
        ]);
        self.buf.extend_from_slice(&imm.to_le_bytes());
        self
    }

    pub fn mov_reg64_imm64(&mut self, dest: RegX64, imm: i64) -> &mut Self {
        self.buf
            .extend_from_slice(&[rex_prefix(true, dest as u8, 0, 0), 0xb8]);
        self.buf.extend_from_slice(&imm.to_le_bytes());
        self
    }

    pub fn mov_ptr64_imm32(&mut self, dest: RegX64, imm: i32) -> &mut Self {
        if let RegX64::RBP | RegX64::R13 = dest {
            return self.mov_ptr64_imm32_disp8(dest, imm, 0);
        };
        self.buf.push(rex_prefix(true, 0, dest as u8, 0));
        self.buf.push(0xc7);
        self.buf.push(mod_rm_byte(NoDisp, 0, dest as u8));
        match dest {
            RegX64::RBP | RegX64::R13 => {
                self.buf.push(0);
            }
            RegX64::RSP | RegX64::R12 => {
                self.buf.push(sib_byte(1, 4, dest as u8));
            }
            _ => (),
        };
        for i in 0..4 {
            self.buf.push((imm >> (8 * i) & 0xff) as u8);
        }
        self
    }

    pub fn mov_ptr64_imm32_disp8(&mut self, dest: RegX64, imm: i32, disp: i8) -> &mut Self {
        self.buf.push(rex_prefix(true, 0, dest as u8, 0));
        self.buf.push(0xc7);
        self.buf.push(mod_rm_byte(Disp8, 0, dest as u8));
        match dest {
            RegX64::RSP | RegX64::R12 => {
                self.buf.push(sib_byte(1, 4, dest as u8));
            }
            _ => (),
        };
        self.buf.push(disp as u8);
        for i in 0..4 {
            self.buf.push((imm >> (8 * i) & 0xff) as u8);
        }
        self
    }

    pub fn ret(&mut self) -> &mut Self {
        self.buf.push(0xc3);
        self
    }

    /*
        Stack operations only used for the host stack (register saving, spilling,  etc.) so they
        use 64-bit register operands
    */

    pub fn push_reg64(&mut self, reg: RegX64) -> &mut Self {
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x50 | (reg as u8 & 0x7));
        self
    }

    pub fn pop_reg64(&mut self, reg: RegX64) -> &mut Self {
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x58 | (reg as u8 & 0x7));
        self
    }

    pub fn push_ptr64(&mut self, reg: RegX64) -> &mut Self {
        if reg == RegX64::RBP || reg == RegX64::R13 {
            return self.push_ptr64_disp8(reg, 0);
        }
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0xff | (reg as u8 & 0x7));
        self.buf.push(mod_rm_byte(NoDisp, 0x6, reg as u8));
        if reg == RegX64::RSP || reg == RegX64::R12 {
            self.buf.push(0x24);
        }
        self
    }

    pub fn pop_ptr64(&mut self, reg: RegX64) -> &mut Self {
        if reg == RegX64::RBP || reg == RegX64::R13 {
            return self.pop_ptr64_disp8(reg, 0);
        }
        if (reg as u8) >= 8 {
            // For extended 64-bit registers (R8-15), reg msb is stored in the REX prefix
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x8f | (reg as u8 & 0x7));
        self.buf.push(mod_rm_byte(NoDisp, 0, reg as u8));
        if reg == RegX64::RSP || reg == RegX64::R12 {
            self.buf.push(sib_byte(1, 4, reg as u8));
        }
        self
    }

    pub fn push_ptr64_disp8(&mut self, reg: RegX64, disp: i8) -> &mut Self {
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0xff | (reg as u8 & 0x7));
        self.buf.push(mod_rm_byte(Disp8, 0x6, reg as u8));
        if reg == RegX64::RSP || reg == RegX64::R12 {
            self.buf.push(sib_byte(1, 4, reg as u8));
        }
        self.buf.push(disp as u8);
        self
    }

    pub fn pop_ptr64_disp8(&mut self, reg: RegX64, disp: i8) -> &mut Self {
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x8f | (reg as u8 & 0x7));
        self.buf.push(mod_rm_byte(Disp8, 0, reg as u8));
        if reg == RegX64::RSP || reg == RegX64::R12 {
            self.buf.push(sib_byte(1, 4, reg as u8));
        }
        self.buf.push(disp as u8);
        self
    }

    pub fn sub_reg64_imm32(&mut self, reg: RegX64, imm: i32) -> &mut Self {
        self.buf.extend_from_slice(&[
            rex_prefix(true, 0, reg as u8, 0),
            0x81,
            mod_rm_byte(Value, 5, reg as u8),
        ]);
        self.buf.extend_from_slice(&imm.to_le_bytes());
        self
    }
}

#[cfg(test)]
mod tests;
