use std::vec::Vec;

#[derive(Copy, Clone, Debug, PartialEq)]
enum RegSize {
    Byte = 8,
    Word = 16,
    Doubleword = 32,
    Quadword = 64,
}
use RegSize::*;

#[derive(Copy, Clone)]
pub struct RegX64 {
    value: u8,
    size: RegSize,
}

impl RegX64 {
    const fn new(value: u8, size: RegSize) -> RegX64 {
        RegX64 { value, size }
    }

    pub fn value(self) -> u8 {
        self.value
    }

    pub fn needs_rex(self) -> bool {
        self.value() > 7
    }

    pub fn msb(self) -> u8 {
        (self.value() >> 4) & 1
    }
    pub fn modrm_bits(self) -> u8 {
        self.value() & 0b111
    }

    pub fn is_16_bit(self) -> bool {
        self.size == Word
    }

    pub fn is_64_bit(self) -> bool {
        self.size == Quadword
    }
}

/// First 2 bits of ModR/M byte. Indicates what kind of displacement follows an instruction when
/// using a register as a pointer, that the register is being used as a value directly
#[derive(Copy, Clone, PartialEq)]
enum Mod {
    NoDisp = 0,
    Disp8 = 1,
    Disp32 = 2,
    Value = 3,
}
use Mod::*;

impl Mod {
    pub fn from_disp(disp: i32) -> Mod {
        match disp {
            d if d == 0 => NoDisp,
            d if -128 <= d && d < 128 => Disp8,
            _ => Disp32,
        }
    }
}

#[derive(Copy, Clone)]
struct Index {
    reg: RegX64,
    scale: u8,
}

#[derive(Copy, Clone)]
pub struct Address {
    base: RegX64,
    op_mod: Mod,
    index: Option<Index>,
    disp: i32,
}

impl Address {
    pub fn displacement(base: RegX64, displacement: i32) -> Address {
        Address {
            base,
            op_mod: Mod::from_disp(displacement),
            index: None,
            disp: displacement,
        }
    }

    pub fn sib(scale: u8, index: RegX64, base: RegX64, displacement: i32) -> Address {
        assert!(scale == 1 || scale == 2 || scale == 3 || scale == 4);
        assert_ne!(index.value(), RSP.value());
        Address {
            base,
            op_mod: Mod::from_disp(displacement),
            index: Some(Index {
                reg: index,
                scale: scale,
            }),
            disp: displacement,
        }
    }
}

/// REX prefix byte, which extends the ModR/M and/or SIB bytes in 64 bit mode by encoding the msb
/// for operands. W indicates 64-bit operands, but is sometimes not needed (e.g. in push/pop)
fn rex_prefix(w: bool, reg: u8, rm_or_base: u8, index: u8) -> u8 {
    let rex = if w { 0x48 } else { 0x40 };
    rex | ((reg >> 3) & 1) << 2 | ((index) >> 3 & 1) << 1 | ((rm_or_base as u8 >> 3) & 1)
}

/// r/m value of b100 in the modr/m byte indicates the scale-index-bound (SIB) addressing mode is
/// used. It also indicates no index when used as the index field of the SIB byte
const SIB_RM: u8 = 0b100;

/// 16-bit override prefix
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

    pub fn add_reg(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        assert_eq!(dest.size, src.size);
        let opcode = if src.size == Byte { 0x00 } else { 0x01 };
        self.emit_modrm_reg(opcode, src, dest)
    }

    pub fn add_addr(&mut self, dest: RegX64, src: Address) -> &mut Self {
        let opcode = if dest.size == Byte { 0x02 } else { 0x03 };
        self.emit_modrm_addr(opcode, dest, src)
    }

    fn emit_rex_addr(&mut self, reg: RegX64, rm: Address) {
        if reg.is_64_bit()
            || reg.needs_rex()
            || rm.base.needs_rex()
            || rm.index.map_or(false, |i| i.reg.needs_rex())
        {
            self.buf.push(rex_prefix(
                reg.is_64_bit(),
                reg.value(),
                rm.base.value(),
                rm.index.map_or(0, |i| i.reg.value()),
            ));
        }
    }

    fn emit_rex_reg(&mut self, reg: RegX64, rm: RegX64) {
        if reg.is_64_bit() || reg.needs_rex() || rm.is_64_bit() || rm.needs_rex() {
            self.buf
                .push(rex_prefix(reg.is_64_bit(), reg.value(), rm.value(), 0));
        }
    }

    fn emit_modrm_reg(&mut self, opcode: u8, reg: RegX64, rm: RegX64) -> &mut Self {
        if reg.is_16_bit() {
            self.buf.push(PREF_16B);
        }
        self.emit_rex_reg(reg, rm);
        self.buf.push(opcode);
        self.buf.push(mod_rm_byte(Value, reg.value(), rm.value()));
        self
    }

    /// Handles logic shared my many instructions (most mov's at least, doesn't support immediate
    /// operands at the moment)
    fn emit_modrm_addr(&mut self, opcode: u8, reg: RegX64, rm: Address) -> &mut Self {
        if reg.is_16_bit() {
            self.buf.push(PREF_16B);
        }
        self.emit_rex_addr(reg, rm);
        self.buf.push(opcode);

        let mut addr = rm;
        // For address with no displacement, RM = 101b (RBP and R13) is used to indicate
        // pc-relative 32-bit offset, so encode as a 0 8bit displacement instead
        if addr.op_mod == NoDisp && (addr.base.modrm_bits() == 0b101) {
            addr.op_mod = Disp8;
            addr.disp = 0;
        }
        // ModR/M byte and SIB byte
        match addr.index {
            None => {
                self.buf
                    .push(mod_rm_byte(addr.op_mod, reg.value(), addr.base.value()));
                // R/M = 100b (RSP and R12) is used to indicate SIB addressing mode, so if one of
                // these is needed as the ptr reg, encode as SIB with no index, (sib = 0x24)
                if addr.base.modrm_bits() == 0b100 {
                    self.buf.push(sib_byte(1, SIB_RM, addr.base.value()));
                }
            }
            Some(index) => {
                // Indicates no index, so cannot be used as an index
                assert!(index.reg.value() != RSP.value());
                self.buf.push(mod_rm_byte(addr.op_mod, reg.value(), SIB_RM));
                self.buf.push(sib_byte(
                    index.scale as u8,
                    index.reg.value(),
                    addr.base.value(),
                ));
            }
        }
        // Displacements
        match addr.op_mod {
            Disp8 => self.buf.push(addr.disp as u8),
            Disp32 => self.buf.extend_from_slice(&addr.disp.to_le_bytes()),
            _ => (),
        };
        self
    }
}

// TODO - Are high bytes needed
pub const AL: RegX64 = RegX64::new(0, Byte);
pub const CL: RegX64 = RegX64::new(1, Byte);
pub const DL: RegX64 = RegX64::new(2, Byte);
pub const BL: RegX64 = RegX64::new(3, Byte);
pub const SPL: RegX64 = RegX64::new(4, Byte);
pub const BPL: RegX64 = RegX64::new(5, Byte);
pub const SIL: RegX64 = RegX64::new(6, Byte);
pub const DIL: RegX64 = RegX64::new(7, Byte);
pub const R8B: RegX64 = RegX64::new(8, Byte);
pub const R9B: RegX64 = RegX64::new(9, Byte);
pub const R10B: RegX64 = RegX64::new(10, Byte);
pub const R11B: RegX64 = RegX64::new(11, Byte);
pub const R12B: RegX64 = RegX64::new(12, Byte);
pub const R13B: RegX64 = RegX64::new(13, Byte);
pub const R14B: RegX64 = RegX64::new(14, Byte);
pub const R15B: RegX64 = RegX64::new(15, Byte);

pub const AX: RegX64 = RegX64::new(0, Word);
pub const CX: RegX64 = RegX64::new(1, Word);
pub const DX: RegX64 = RegX64::new(2, Word);
pub const BX: RegX64 = RegX64::new(3, Word);
pub const SP: RegX64 = RegX64::new(4, Word);
pub const BP: RegX64 = RegX64::new(5, Word);
pub const SI: RegX64 = RegX64::new(6, Word);
pub const DI: RegX64 = RegX64::new(7, Word);
pub const R8W: RegX64 = RegX64::new(8, Word);
pub const R9W: RegX64 = RegX64::new(9, Word);
pub const R10W: RegX64 = RegX64::new(10, Word);
pub const R11W: RegX64 = RegX64::new(11, Word);
pub const R12W: RegX64 = RegX64::new(12, Word);
pub const R13W: RegX64 = RegX64::new(13, Word);
pub const R14W: RegX64 = RegX64::new(14, Word);
pub const R15W: RegX64 = RegX64::new(15, Word);

pub const EAX: RegX64 = RegX64::new(0, Doubleword);
pub const ECX: RegX64 = RegX64::new(1, Doubleword);
pub const EDX: RegX64 = RegX64::new(2, Doubleword);
pub const EBX: RegX64 = RegX64::new(3, Doubleword);
pub const ESP: RegX64 = RegX64::new(4, Doubleword);
pub const EBP: RegX64 = RegX64::new(5, Doubleword);
pub const ESI: RegX64 = RegX64::new(6, Doubleword);
pub const EDI: RegX64 = RegX64::new(7, Doubleword);
pub const R8D: RegX64 = RegX64::new(8, Doubleword);
pub const R9D: RegX64 = RegX64::new(9, Doubleword);
pub const R10D: RegX64 = RegX64::new(10, Doubleword);
pub const R11D: RegX64 = RegX64::new(11, Doubleword);
pub const R12D: RegX64 = RegX64::new(12, Doubleword);
pub const R13D: RegX64 = RegX64::new(13, Doubleword);
pub const R14D: RegX64 = RegX64::new(14, Doubleword);
pub const R15D: RegX64 = RegX64::new(15, Doubleword);

pub const RAX: RegX64 = RegX64::new(0, Quadword);
pub const RCX: RegX64 = RegX64::new(1, Quadword);
pub const RDX: RegX64 = RegX64::new(2, Quadword);
pub const RBX: RegX64 = RegX64::new(3, Quadword);
pub const RSP: RegX64 = RegX64::new(4, Quadword);
pub const RBP: RegX64 = RegX64::new(5, Quadword);
pub const RSI: RegX64 = RegX64::new(6, Quadword);
pub const RDI: RegX64 = RegX64::new(7, Quadword);
pub const R8: RegX64 = RegX64::new(8, Quadword);
pub const R9: RegX64 = RegX64::new(9, Quadword);
pub const R10: RegX64 = RegX64::new(10, Quadword);
pub const R11: RegX64 = RegX64::new(11, Quadword);
pub const R12: RegX64 = RegX64::new(12, Quadword);
pub const R13: RegX64 = RegX64::new(13, Quadword);
pub const R14: RegX64 = RegX64::new(14, Quadword);
pub const R15: RegX64 = RegX64::new(15, Quadword);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_reg32_reg32() {
        let mut code = EmitterX64::new();
        code.add_reg(EBX, EBP)
            .add_reg(EAX, R15D)
            .add_reg(R11D, ESI)
            .add_reg(R8D, R9D);
        assert_eq!(
            code.buf,
            vec![
                0x01, 0xEB, // add ebx, ebp
                0x44, 0x01, 0xF8, // add eax, r15d
                0x41, 0x01, 0xF3, // add r11d, esi
                0x45, 0x01, 0xC8, // add r8d, r9d
            ]
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_add_reg32_ptr64_disp8() {
        let mut code = EmitterX64::new();
        code.add_addr(EBX, Address::displacement(RBP, 12))
            .add_addr(EAX, Address::displacement(R12,-128))
            .add_addr(R11D, Address::displacement(RSP, 90))
            .add_addr(R8D, Address::displacement(R13, 127))
            .add_addr(EDX, Address::displacement(RCX, 70));
        assert_eq!(
            code.buf,
            vec![
                0x03, 0x5D, 0x0C, // add ebx, [rbp+12]
                0x41, 0x03, 0x44, 0x24, 0x80, // add eax, [r12-128]
                0x44, 0x03, 0x5C, 0x24, 0x5A, // add r11d, [rsp+90]
                0x45, 0x03, 0x45, 0x7F, // add r8d,[r13+127]
                0x03, 0x51, 0x46, // add edx, [rcx+70]
            ]
        );
}
}
