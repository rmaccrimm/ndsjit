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

#[derive(Copy, Clone, PartialEq)]
pub enum RegOperand {
    Reg8(RegX64),
    Reg16(RegX64),
    Reg32(RegX64),
    Reg64(RegX64),
}
use RegOperand::*;

impl RegOperand {
    pub fn is_reg8(&self) -> bool {
        match self {
            Reg8(_) => true,
            _ => false,
        }
    }

    pub fn is_reg16(&self) -> bool {
        match self {
            Reg16(_) => true,
            _ => false,
        }
    }

    pub fn is_reg32(&self) -> bool {
        match self {
            Reg32(_) => true,
            _ => false,
        }
    }

    pub fn is_reg64(&self) -> bool {
        match self {
            Reg64(_) => true,
            _ => false,
        }
    }

    pub fn unwrap(&self) -> RegX64 {
        match self {
            Reg8(r) | Reg16(r) | Reg32(r) | Reg64(r) => *r,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PtrOperand {
    BaseNoDisp {
        base: RegX64,
    },
    BaseDisp8 {
        base: RegX64,
        disp: u8,
    },
    BaseDisp32 {
        base: RegX64,
        disp: u32,
    },
    SIBNoDisp {
        base: RegX64,
        index: RegX64,
        scale: u8,
    },
    SIBDisp8 {
        base: RegX64,
        index: RegX64,
        scale: u8,
        disp: u8,
    },
    SIBDisp32 {
        base: RegX64,
        index: RegX64,
        scale: u8,
        disp: u32,
    },
    /// Not really a pointer, but encoded the same way.
    /// Indicates the register value is being used directly
    RegValue {
        base: RegX64,
    },
}
use PtrOperand::*;

impl PtrOperand {
    /// Get base register as u8
    pub fn base(&self) -> u8 {
        match self {
            BaseNoDisp { base }
            | BaseDisp8 { base, .. }
            | BaseDisp32 { base, .. }
            | SIBNoDisp { base, .. }
            | SIBDisp8 { base, .. }
            | SIBDisp32 { base, .. }
            | RegValue { base } => *base as u8,
        }
    }

    /// Get index register as u8, if present, or return default
    pub fn ind_or(&self, default: u8) -> u8 {
        match self {
            BaseNoDisp { .. } | BaseDisp8 { .. } | BaseDisp32 { .. } | RegValue { .. } => default,
            SIBNoDisp { index, .. } | SIBDisp8 { index, .. } | SIBDisp32 { index, .. } => {
                *index as u8
            }
        }
    }

    /// Get mod bits for operand
    pub fn op_mod(&self) -> Mod {
        match self {
            BaseNoDisp { .. } | SIBNoDisp { .. } => NoDisp,
            BaseDisp8 { .. } | SIBDisp8 { .. } => Disp8,
            BaseDisp32 { .. } | SIBDisp32 { .. } => Disp32,
            RegValue { .. } => Value,
        }
    }
}

/// First 2 bits of ModR/M byte. Indicates what kind of displacement follows an instruction when
/// using a register as a pointer, that the register is being used as a value directly
#[derive(Copy, Clone, PartialEq)]
pub enum Mod {
    NoDisp = 0,
    Disp8 = 1,
    Disp32 = 2,
    Value = 3,
}
use Mod::*;

/// REX prefix byte, which extends the ModR/M and/or SIB bytes in 64 bit mode by encoding the msb
/// for operands. W indicates 64-bit operands, but is sometimes not needed (e.g. in push/pop)
fn rex_prefix(w: bool, reg: u8, rm_or_base: u8, index: u8) -> u8 {
    let rex = if w { 0x48 } else { 0x40 };
    rex | ((reg >> 3) & 1) << 2 | ((index) >> 3 & 1) << 1 | ((rm_or_base as u8 >> 3) & 1)
}

/// r/m value of b100 in the modr/m byte indicates the scale-index-bound (SIB) addressing mode is
/// used. It also indicates no index when used as the index field of the SIB byte
const SIB_RM: u8 = 0b100;

const PREF_16B: u8 = 0x66;

/// ModR/M byte which encodes an addressing mode, and the 3 lsb of a register operand (reg) and
/// register or memory operand (r/m). Alternatively the reg field is sometimes an extension of the
/// instruction opcode
fn mod_rm_byte(mode: Mod, reg_or_op: u8, rm: u8) -> u8 {
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

impl EmitterX64 {
    pub fn new() -> EmitterX64 {
        EmitterX64 { buf: Vec::new() }
    }

    /// Handles logic shared my many instructions (most mov's at least, doesn't support immediate
    /// operands at the moment)
    fn modrm_instr(&mut self, opcode: u8, reg: RegOperand, ptr: PtrOperand) -> &mut Self {
        // 16-bit override prefix
        if reg.is_reg16() {
            self.buf.push(PREF_16B);
        }
        // REX prefix
        if reg.is_reg64() || reg.unwrap() as u8 > 7 || ptr.base() > 7 || ptr.ind_or(0) > 7 {
            self.buf.push(rex_prefix(
                reg.is_reg64(),
                reg.unwrap() as u8,
                ptr.base(),
                ptr.ind_or(0),
            ))
        }
        self.buf.push(opcode);
        let mut ptr = ptr;

        // For ptr operand with no displacement, RM = 101b (RBP and R13) is used to indicate
        // pc-relative 32-bit offset, so encode as a 0 8bit displacement instead
        if ptr.op_mod() == NoDisp && (ptr.base() & 0b111 == 0b101) {
            ptr = match ptr {
                BaseNoDisp { base } => BaseDisp8 { base, disp: 0 },
                SIBNoDisp { base, index, scale } => SIBDisp8 {
                    base,
                    index,
                    scale,
                    disp: 0,
                },
                _ => ptr,
            }
        }
        // ModR/M byte and SIB byte
        match ptr {
            RegValue { base } => {
                self.buf
                    .push(mod_rm_byte(ptr.op_mod(), reg.unwrap() as u8, base as u8));
            }
            BaseNoDisp { base } | BaseDisp8 { base, .. } | BaseDisp32 { base, .. } => {
                self.buf
                    .push(mod_rm_byte(ptr.op_mod(), reg.unwrap() as u8, base as u8));
                // R/M = 100b (RSP and R12) is used to indicate SIB addressing mode, so if one of
                // these is needed as the ptr reg, encode as SIB with no index, (sib = 0x24)
                if ptr.base() & 0b111 == 0b100 {
                    self.buf.push(sib_byte(1, SIB_RM, base as u8));
                }
            }
            SIBNoDisp { base, index, scale }
            | SIBDisp8 {
                base, index, scale, ..
            }
            | SIBDisp32 {
                base, index, scale, ..
            } => {
                // Indicates no index, so cannot be used as an index
                assert!(index != RegX64::RSP);
                self.buf
                    .push(mod_rm_byte(ptr.op_mod(), reg.unwrap() as u8, SIB_RM));
                self.buf.push(sib_byte(scale, index as u8, base as u8));
            }
        }
        // Displacements
        match ptr {
            BaseDisp8 { disp, .. } | SIBDisp8 { disp, .. } => self.buf.push(disp as u8),
            BaseDisp32 { disp, .. } | SIBDisp32 { disp, .. } => {
                self.buf.extend_from_slice(&disp.to_le_bytes())
            }
            _ => (),
        };
        self
    }

    /// add %r32>, %r32>
    pub fn add_reg32_reg32(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.modrm_instr(0x01, Reg32(src), RegValue { base: dest })
    }

    /// add %r32, [%r64 + i8]
    pub fn add_reg32_ptr64_disp8(&mut self, dest: RegX64, src: RegX64, disp: i8) -> &mut Self {
        self.modrm_instr(
            0x03,
            Reg32(dest),
            BaseDisp8 {
                base: src,
                disp: disp as u8,
            },
        )
    }

    /// call %r64
    pub fn call_reg64(&mut self, reg: RegX64) -> &mut Self {
        self.buf
            .extend_from_slice(&[0xff, mod_rm_byte(Value, 2, reg as u8)]);
        self
    }

    /// mov %r64, %r64
    pub fn mov_reg64_reg64(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.modrm_instr(0x89, Reg64(src), RegValue { base: dest })
    }

    /// mov %r32, %r32
    pub fn mov_reg32_reg32(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.modrm_instr(0x89, Reg32(src), RegValue { base: dest })
    }

    /// mov %r32, [%r64]
    pub fn mov_reg_ptr(&mut self, dest: RegOperand, src: PtrOperand) -> &mut Self {
        if dest.is_reg8() {
            self.modrm_instr(0x8a, dest, src)
        } else {
            self.modrm_instr(0x8b, dest, src)
        }
    }

    /// mov [%r64], %r32
    pub fn mov_ptr64_reg32(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        self.modrm_instr(0x89, Reg32(src), BaseNoDisp { base: dest })
    }

    /// mov %r32, [%r64 + i8]
    pub fn mov_reg32_ptr64_disp8(&mut self, dest: RegX64, src: RegX64, disp: i8) -> &mut Self {
        self.modrm_instr(
            0x8b,
            Reg32(dest),
            BaseDisp8 {
                base: src,
                disp: disp as u8,
            },
        )
    }

    /// mov [%r64 + i8], %r32
    pub fn mov_ptr64_reg32_disp8(&mut self, dest: RegX64, src: RegX64, disp: i8) -> &mut Self {
        self.modrm_instr(
            0x89,
            Reg32(src),
            BaseDisp8 {
                base: dest,
                disp: disp as u8,
            },
        )
    }

    /// mov %r32, [%r64 + i32]
    pub fn mov_reg32_ptr64_disp32(&mut self, dest: RegX64, src: RegX64, disp: i32) -> &mut Self {
        self.modrm_instr(
            0x8b,
            Reg32(dest),
            BaseDisp32 {
                base: src,
                disp: disp as u32,
            },
        )
    }

    /// mov [%r64 + i8], %r32
    pub fn mov_ptr64_reg32_disp32(&mut self, dest: RegX64, src: RegX64, disp: i32) -> &mut Self {
        self.modrm_instr(
            0x89,
            Reg32(src),
            BaseDisp32 {
                base: dest,
                disp: disp as u32,
            },
        )
    }

    /// mov %r32, [%r64 + scale * %r64]
    pub fn mov_reg32_ptr64_sib(
        &mut self,
        dest: RegX64,
        base: RegX64,
        scale: u8,
        ind: RegX64,
    ) -> &mut Self {
        self.modrm_instr(
            0x8b,
            Reg32(dest),
            SIBNoDisp {
                base,
                scale,
                index: ind,
            },
        )
    }

    /// mov %r32, [%r64 + scale * %r64 + i32]
    pub fn mov_reg32_ptr64_sib_disp32(
        &mut self,
        dest: RegX64,
        base: RegX64,
        scale: u8,
        ind: RegX64,
        disp: i32,
    ) -> &mut Self {
        self.modrm_instr(
            0x8b,
            Reg32(dest),
            SIBDisp32 {
                base,
                scale,
                index: ind,
                disp: disp as u32,
            },
        )
    }

    /// mov %r64, i32
    pub fn mov_reg64_imm32(&mut self, dest: RegX64, imm: i32) -> &mut Self {
        self.buf.extend_from_slice(&[
            rex_prefix(true, 0, dest as u8, 0),
            0xc7,
            mod_rm_byte(Value, 0, dest as u8),
        ]);
        self.buf.extend_from_slice(&imm.to_le_bytes());
        self
    }

    /// mov %r64, i64
    pub fn mov_reg64_imm64(&mut self, dest: RegX64, imm: i64) -> &mut Self {
        self.buf
            .extend_from_slice(&[rex_prefix(true, dest as u8, 0, 0), 0xb8]);
        self.buf.extend_from_slice(&imm.to_le_bytes());
        self
    }

    /// mov [%r64], i32
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

    /// mov [%r64 + i8], i32
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

    /// push %r64
    pub fn push_reg64(&mut self, reg: RegX64) -> &mut Self {
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x50 | (reg as u8 & 0x7));
        self
    }

    /// pop %r64
    pub fn pop_reg64(&mut self, reg: RegX64) -> &mut Self {
        if (reg as u8) >= 8 {
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x58 | (reg as u8 & 0x7));
        self
    }

    /// push [%r64]
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

    /// pop [%r64]
    pub fn pop_ptr64(&mut self, reg: RegX64) -> &mut Self {
        if reg == RegX64::RBP || reg == RegX64::R13 {
            return self.pop_ptr64_disp8(reg, 0);
        }
        if (reg as u8) >= 8 {
            // For extended 64-bit registers (Reg8-15), reg msb is stored in the REX prefix
            self.buf.push(rex_prefix(false, 0, reg as u8, 0));
        }
        self.buf.push(0x8f | (reg as u8 & 0x7));
        self.buf.push(mod_rm_byte(NoDisp, 0, reg as u8));
        if reg == RegX64::RSP || reg == RegX64::R12 {
            self.buf.push(sib_byte(1, 4, reg as u8));
        }
        self
    }

    /// push [%r64 + i8]
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

    /// pop [%r64 + i8]
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

    /// sub %r64, i32
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
