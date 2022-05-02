use std::vec::Vec;

#[derive(Copy, Clone, Debug, PartialEq)]
enum OperandSize {
    Byte = 8,
    Word = 16,
    Doubleword = 32,
    Quadword = 64,
}
use OperandSize::*;

#[derive(Copy, Clone)]
pub struct RegX64 {
    value: u8,
    size: OperandSize,
}

impl RegX64 {
    const fn new(value: u8, size: OperandSize) -> RegX64 {
        assert!(value < 16);
        RegX64 { value, size }
    }

    pub fn reg32(value: u8) -> RegX64 {
        assert!(value < 16);
        RegX64 {
            value,
            size: Doubleword,
        }
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
    pub fn disp(base: RegX64, displacement: i32) -> Address {
        Address {
            base,
            op_mod: Mod::from_disp(displacement),
            index: None,
            disp: displacement,
        }
    }

    pub fn sib(scale: u8, index: RegX64, base: RegX64, displacement: i32) -> Address {
        assert!(scale == 1 || scale == 2 || scale == 4 || scale == 8);
        assert!(index.size == base.size);
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

const PC_REL_RM: u8 = 0b101;

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

    pub fn hex_dump(&self) {
        for b in self.buf.iter() {
            print!("{:02x}", b);
        }
        println!();
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

    pub fn mov_reg_reg(&mut self, dest: RegX64, src: RegX64) -> &mut Self {
        assert_eq!(dest.size, src.size);
        let opcode = if src.size == Byte { 0x88 } else { 0x89 };
        self.emit_modrm_reg(opcode, src, dest)
    }

    pub fn mov_reg_addr(&mut self, dest: RegX64, src: Address) -> &mut Self {
        let opcode = if dest.size == Byte { 0x8a } else { 0x8b };
        self.emit_modrm_addr(opcode, dest, src)
    }

    pub fn mov_addr_reg(&mut self, dest: Address, src: RegX64) -> &mut Self {
        let opcode = if src.size == Byte { 0x88 } else { 0x89 };
        self.emit_modrm_addr(0x89, src, dest)
    }

    pub fn mov_reg_imm(&mut self, dest: RegX64, imm: i64) -> &mut Self {
        let opcode = if dest.size == Byte { 0xb9 } else { 0xb8 };
        self.emit_rex_reg(RegX64::new(0, Doubleword), dest);
        self.buf.push(opcode | dest.modrm_bits());
        self.emit_imm_data(imm, dest.size)
    }

    pub fn mov_addr_imm32(&mut self, dest: Address, imm: i32) -> &mut Self {
        self.emit_modrm_addr(0xc7, RegX64::new(0, Quadword), dest);
        self.buf.extend_from_slice(&imm.to_le_bytes());
        self
    }

    pub fn push_reg(&mut self, reg: RegX64) -> &mut Self {
        if reg.needs_rex() {
            self.buf.push(rex_prefix(false, 0, reg.value(), 0));
        }
        self.buf.push(0x50 | reg.modrm_bits());
        self
    }

    pub fn push_addr(&mut self, addr: Address) -> &mut Self {
        self.emit_modrm_addr(0xff, RegX64::new(6, Doubleword), addr)
    }

    pub fn pop_reg(&mut self, reg: RegX64) -> &mut Self {
        if reg.needs_rex() {
            self.buf.push(rex_prefix(false, 0, reg.value(), 0));
        }
        self.buf.push(0x58 | reg.modrm_bits());
        self
    }

    pub fn pop_addr(&mut self, addr: Address) -> &mut Self {
        self.emit_modrm_addr(0x8f, RegX64::new(0, Doubleword), addr)
    }

    pub fn ret(&mut self) -> &mut Self {
        self.buf.push(0xc3);
        self
    }

    pub fn sub_reg_imm32(&mut self, reg: RegX64, imm: i32) -> &mut Self {
        self.emit_modrm_reg(0x81, RegX64::new(0x5, Quadword), reg);
        self.emit_imm_data(imm as i64, Doubleword)
    }

    fn emit_imm_data(&mut self, imm: i64, size: OperandSize) -> &mut Self {
        match size {
            Byte => self.buf.extend_from_slice(&(imm as i8).to_le_bytes()),
            Word => self.buf.extend_from_slice(&(imm as i16).to_le_bytes()),
            Doubleword => self.buf.extend_from_slice(&(imm as i32).to_le_bytes()),
            Quadword => self.buf.extend_from_slice(&imm.to_le_bytes()),
        };
        self
    }

    fn emit_rex_addr(&mut self, reg: RegX64, rm: Address) {
        if reg.size == Quadword
            || reg.needs_rex()
            || rm.base.needs_rex()
            || rm.index.map_or(false, |i| i.reg.needs_rex())
        {
            self.buf.push(rex_prefix(
                reg.size == Quadword,
                reg.value(),
                rm.base.value(),
                rm.index.map_or(0, |i| i.reg.value()),
            ));
        }
    }

    fn emit_rex_reg(&mut self, reg: RegX64, rm: RegX64) {
        if reg.size == Quadword || reg.needs_rex() || rm.size == Quadword || rm.needs_rex() {
            self.buf.push(rex_prefix(
                reg.size == Quadword || rm.size == Quadword,
                reg.value(),
                rm.value(),
                0,
            ));
        }
    }

    /// Emit a ModR/M encoded instruction with two register operands
    fn emit_modrm_reg(&mut self, opcode: u8, reg: RegX64, rm: RegX64) -> &mut Self {
        if reg.size == Word {
            self.buf.push(PREF_16B);
        }
        self.emit_rex_reg(reg, rm);
        self.buf.push(opcode);
        self.buf.push(mod_rm_byte(Value, reg.value(), rm.value()));
        self
    }

    /// Emit a ModR/M encoded instruction with an address operand
    fn emit_modrm_addr(&mut self, opcode: u8, reg: RegX64, rm: Address) -> &mut Self {
        if reg.size == Word {
            self.buf.push(PREF_16B);
        }
        self.emit_rex_addr(reg, rm);
        self.buf.push(opcode);

        let mut addr = rm;
        // For address with no displacement, RM = 101b (RBP and R13) is used to indicate
        // pc-relative 32-bit offset, so encode as a 0 8bit displacement instead
        if addr.op_mod == NoDisp && (addr.base.modrm_bits() == PC_REL_RM) {
            addr.op_mod = Disp8;
            addr.disp = 0;
        }
        // ModR/M byte and SIB byte
        match addr.index {
            None => {
                self.buf.push(mod_rm_byte(addr.op_mod, reg.value(), addr.base.value()));
                // R/M = 100b (RSP and R12) is used to indicate SIB addressing mode, so if one of
                // these is needed as the ptr reg, encode as SIB with no index, (sib = 0x24)
                if addr.base.modrm_bits() == SIB_RM {
                    self.buf.push(sib_byte(1, SIB_RM, addr.base.value()));
                }
            }
            Some(index) => {
                // Indicates no index, so cannot be used as an index
                assert!(index.reg.value() != RSP.value());
                self.buf.push(mod_rm_byte(addr.op_mod, reg.value(), SIB_RM));
                self.buf.push(sib_byte(index.scale as u8, index.reg.value(), addr.base.value()));
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
#[rustfmt::skip]
mod tests {
    use super::*;

    macro_rules! assert_emit_eq {
        ($method:ident ($($param:expr),*), $($e:expr),*) => {
            let mut code = EmitterX64::new();
            code.$method($($param),*);

            let mut emit_str:  Vec<String> = Vec::new();
            for x in code.buf.iter() {
                emit_str.push(format!("{:02x}", x));
            }

            let mut exp_str: Vec<String> = Vec::new();
            $(
                exp_str.push(format!("{:02x}", $e));
            )*

            assert!(
                code.buf == vec![$($e),*],
                "assertion failed: Emitted code did not match expected\n \
                  emitted: [{}]\n\
                 expected: [{}]\n",
                emit_str.join(", "),
                exp_str.join(", ")
            );
        };
    }

    #[test]
    fn test_add_reg32_reg32() {
        assert_emit_eq!(add_reg(EBX, EBP), 0x01, 0xEB);
        assert_emit_eq!(add_reg(EAX, R15D), 0x44, 0x01, 0xF8);
        assert_emit_eq!(add_reg(R11D, ESI), 0x41, 0x01, 0xF3);
        assert_emit_eq!(add_reg(R8D, R9D), 0x45, 0x01, 0xC8);
    }

    #[test]
    fn test_add_reg32_addr64_disp8() {
        assert_emit_eq!(add_addr(EBX, Address::disp(RBP, 12)), 0x03, 0x5D, 0x0C);
        assert_emit_eq!(add_addr(EAX, Address::disp(R12, -128)), 0x41, 0x03, 0x44, 0x24, 0x80);
        assert_emit_eq!(add_addr(R11D, Address::disp(RSP, 90)), 0x44, 0x03, 0x5C, 0x24, 0x5A);
        assert_emit_eq!(add_addr(R8D, Address::disp(R13, 127)), 0x45, 0x03, 0x45, 0x7F);
        assert_emit_eq!(add_addr(EDX, Address::disp(RCX, 70)), 0x03, 0x51, 0x46);
    }

    #[test]
    fn test_mov_reg32_reg32() {
        assert_emit_eq!(mov_reg_reg(EAX, R15D), 0x44, 0x89, 0xF8);
        assert_emit_eq!(mov_reg_reg(ESP, EBP), 0x89, 0xEC);
        assert_emit_eq!(mov_reg_reg(EBX, R9D), 0x44, 0x89, 0xCB);
    }

    #[test]
    fn test_mov_reg64_reg64() {
        assert_emit_eq!(mov_reg_reg(RBX, RDX), 0x48, 0x89, 0xD3);
        assert_emit_eq!(mov_reg_reg(RDX, RBP), 0x48, 0x89, 0xEA);
        assert_emit_eq!(mov_reg_reg(R9, RSP), 0x49, 0x89, 0xE1);
        assert_emit_eq!(mov_reg_reg(RCX, R12), 0x4C, 0x89, 0xE1);
    }

    #[test]
    #[should_panic]
    fn test_mov_reg_different_sizes() {
        let mut code = EmitterX64::new();
        code.mov_reg_reg(EAX, R12);
    }

    #[test]
    fn test_mov_reg32_addr64() {
        assert_emit_eq!(mov_reg_addr(R8D, Address::disp(RBP, 0)), 0x44, 0x8B, 0x45, 0x00);
        assert_emit_eq!(mov_reg_addr(R15D, Address::disp(RSI, 0)), 0x44, 0x8B, 0x3E);
        assert_emit_eq!(mov_reg_addr(EDI, Address::disp(RBX, 0)), 0x8B, 0x3B);
        assert_emit_eq!(mov_reg_addr(EAX, Address::disp(RAX, 0)), 0x8B, 0x00);
        assert_emit_eq!(mov_reg_addr(R11D, Address::disp(RCX, 0)), 0x44, 0x8B, 0x19);
        assert_emit_eq!(mov_reg_addr(EBP, Address::disp(RSP, 0)), 0x8B, 0x2C, 0x24);
        assert_emit_eq!(mov_reg_addr(ECX, Address::disp(RDI, 0)), 0x8B, 0x0F);
        assert_emit_eq!(mov_reg_addr(R9D, Address::disp(R12, 0)), 0x45, 0x8B, 0x0C, 0x24);
        assert_emit_eq!(mov_reg_addr(EAX, Address::disp(R13, 0)), 0x41, 0x8B, 0x45, 0x00);
    }

    #[test]
    fn test_mov_addr64_reg32() {
        assert_emit_eq!(mov_addr_reg(Address::disp(RBP, 0), EDI), 0x89, 0x7D, 0x00);
        assert_emit_eq!(mov_addr_reg(Address::disp(RSP, 0), EAX), 0x89, 0x04, 0x24);
        assert_emit_eq!(mov_addr_reg(Address::disp(R12, 0), R15D), 0x45, 0x89, 0x3C, 0x24);
        assert_emit_eq!(mov_addr_reg(Address::disp(R13, 0), R13D), 0x45, 0x89, 0x6D, 0x00);
    }

    #[test]
    fn test_mov_reg32_addr64_disp8() {
        assert_emit_eq!(mov_reg_addr(R8D, Address::disp(RBP, 127)), 0x44, 0x8B, 0x45, 0x7F);
        assert_emit_eq!(mov_reg_addr(R9D, Address::disp(RSP, 10)), 0x44, 0x8B, 0x4C, 0x24, 0x0A);
        assert_emit_eq!(mov_reg_addr(R10D, Address::disp(R12, 99)), 0x45, 0x8B, 0x54, 0x24, 0x63);
        assert_emit_eq!(mov_reg_addr(R11D, Address::disp(R13, -45)), 0x45, 0x8B, 0x5D, 0xD3);
        assert_emit_eq!(mov_reg_addr(ECX, Address::disp(R15, 109)), 0x41, 0x8B, 0x4F, 0x6D);
        assert_emit_eq!(mov_reg_addr(EBX, Address::disp(RAX, 12)), 0x8B, 0x58, 0x0C);
    }

    #[test]
    fn test_mov_addr64_reg32_disp8() {
        assert_emit_eq!(mov_addr_reg(Address::disp(RBP, -78), EAX), 0x89, 0x45, 0xB2);
        assert_emit_eq!(mov_addr_reg(Address::disp(RSP, 10), EBX), 0x89, 0x5C, 0x24, 0x0A);
        assert_emit_eq!(mov_addr_reg(Address::disp(R12, -3), ECX), 0x41, 0x89, 0x4C, 0x24, 0xFD);
        assert_emit_eq!(mov_addr_reg(Address::disp(R13, 44), R15D), 0x45, 0x89, 0x7D, 0x2C);
        assert_emit_eq!(mov_addr_reg(Address::disp(RDI, -1), ESI), 0x89, 0x77, 0xFF);
    }

    #[test]
    fn test_mov_reg32_addr64_disp32() {
        assert_emit_eq!(
            mov_reg_addr(EBX, Address::disp(RSP, 16000)), 
            0x8B, 0x9C, 0x24, 0x80, 0x3E, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_reg_addr(ESP, Address::disp(RBP, 453)), 
            0x8B, 0xA5, 0xC5, 0x01, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_reg_addr(R14D, Address::disp(R12, -883)),
            0x45, 0x8B, 0xB4, 0x24, 0x8D, 0xFC, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_reg_addr(ESI, Address::disp(R13, -10000)), 
            0x41, 0x8B, 0xB5, 0xF0, 0xD8, 0xFF, 0xFF
        );
    }

    #[test]
    fn test_mov_addr64_reg32_disp32() {
        assert_emit_eq!(
            mov_addr_reg(Address::disp(RSP, 16000), R11D),
            0x44, 0x89, 0x9C, 0x24, 0x80, 0x3E, 0x00, 0x00
        ); 
        assert_emit_eq!(
            mov_addr_reg(Address::disp(RBP, 453), EAX),
            0x89, 0x85, 0xC5, 0x01, 0x00, 0x00
        ); 
        assert_emit_eq!(
            mov_addr_reg(Address::disp(R12, -883), EDI),
            0x41, 0x89, 0xBC, 0x24, 0x8D, 0xFC, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_addr_reg(Address::disp(R13, -10000), ECX),
            0x41, 0x89, 0x8D, 0xF0, 0xD8, 0xFF, 0xFF
        );
    }

    #[test]
    fn test_mov_reg32_imm32() {
        assert_emit_eq!(mov_reg_imm(EAX, 485884), 0xB8, 0xFC, 0x69, 0x07, 0x00);
        assert_emit_eq!(mov_reg_imm(EBP, 0), 0xBD, 0x00, 0x00, 0x00, 0x00);
        assert_emit_eq!(mov_reg_imm(ESP, 19), 0xBC, 0x13, 0x00, 0x00, 0x00);
        assert_emit_eq!(mov_reg_imm(R12D, 753432), 0x41, 0xBC, 0x18, 0x7F, 0x0B, 0x00);
        assert_emit_eq!(mov_reg_imm(R13D, 458), 0x41, 0xBD, 0xCA, 0x01, 0x00, 0x00);
        assert_emit_eq!(mov_reg_imm(R15D, 2147483647), 0x41, 0xBF, 0xFF, 0xFF, 0xFF, 0x7F);
        assert_emit_eq!(mov_reg_imm(ESI, -28654), 0xBE, 0x12, 0x90, 0xFF, 0xFF);
    }

    #[test]
    fn test_mov_reg64_imm64() {
        assert_emit_eq!(
            mov_reg_imm(RAX, 500000000000),
            0x48, 0xB8, 0x00, 0x88, 0x52, 0x6A, 0x74, 0x00, 0x00, 0x00
        );
    }

    #[test]
    fn test_mov_addr_imm32() {
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(RCX, 0), -98),
            0x48, 0xC7, 0x01, 0x9E, 0xFF, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(RBP, 0), 127),
            0x48, 0xC7, 0x45, 0x00, 0x7F, 0x00, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(RSP, 0), -128),
            0x48, 0xC7, 0x04, 0x24, 0x80, 0xFF, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(R12, 0), -0),
            0x49, 0xC7, 0x04, 0x24, 0x00, 0x00, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(R13, 0), 99),
            0x49, 0xC7, 0x45, 0x00, 0x63, 0x00, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(R11, 0), 2),
            0x49, 0xC7, 0x03, 0x02, 0x00, 0x00, 0x00
        );
    }

    #[test]
    fn test_mov_addr_imm32_disp8() {
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(RDX, -10), -98),
            0x48, 0xC7, 0x42, 0xF6, 0x9E, 0xFF, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(RBP, 12), 127),
            0x48, 0xC7, 0x45, 0x0C, 0x7F, 0x00, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(RSP, -9), 2383839),
            0x48, 0xC7, 0x44, 0x24, 0xF7, 0xDF, 0x5F, 0x24, 0x00
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(R12, 1), -129484),
            0x49, 0xC7, 0x44, 0x24, 0x01, 0x34, 0x06, 0xFE, 0xFF
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(R13, 127), 88),
            0x49, 0xC7, 0x45, 0x7F, 0x58, 0x00, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_addr_imm32(Address::disp(R8, 16), 0),
            0x49, 0xC7, 0x40, 0x10, 0x00, 0x00, 0x00, 0x00
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_mov_reg32_addr_sib_disp32() {
        assert_emit_eq!(
            mov_reg_addr(ECX, Address::sib(2, RAX, RBX, 128)),
            0x8B, 0x8C, 0x43, 0x80, 0x00, 0x00, 0x00
        );
        assert_emit_eq!(
            mov_reg_addr(ESI, Address::sib(4, RBP, RBP, -454)),
            0x8B, 0xB4, 0xAD, 0x3A, 0xFE, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_reg_addr(R12D, Address::sib(8, R13, RSP, 209384)),
            0x46, 0x8B, 0xA4, 0xEC, 0xE8, 0x31, 0x03, 0x00
        );
        assert_emit_eq!(
            mov_reg_addr(EAX, Address::sib(1, R12, RDI, -943949)),
            0x42, 0x8B, 0x84, 0x27, 0xB3, 0x98, 0xF1, 0xFF
        );
        assert_emit_eq!(
            mov_reg_addr(ESP, Address::sib(1, R8, R13, -129)),
            0x43, 0x8B, 0xA4, 0x05, 0x7F, 0xFF, 0xFF, 0xFF
        );
        assert_emit_eq!(
            mov_reg_addr(R15D, Address::sib(1, RBX, R12, 349999)),
            0x45, 0x8B, 0xBC, 0x1C, 0x2F, 0x57, 0x05, 0x00
        );
    }

    #[test]
    #[should_panic]
    #[rustfmt::skip]
    fn test_mov_reg32_addr_sib_disp32_sp_index() {
        let mut code = EmitterX64::new();
        code.mov_reg_addr(ECX, Address::sib(2, RSP, RBX, 128));
    }

    #[test]
    #[rustfmt::skip]
    fn test_mov_reg32_addr_sib() {
        assert_emit_eq!(mov_reg_addr(ECX, Address::sib(2, RAX, RBX, 0)), 0x8B, 0x0C, 0x43);
        assert_emit_eq!(mov_reg_addr(ESI, Address::sib(4, RBP, RBP, 0)), 0x8B, 0x74, 0xAD, 0x00);
        assert_emit_eq!(mov_reg_addr(R12D, Address::sib(8, R13, RSP, 0)), 0x46, 0x8B, 0x24, 0xEC);
        assert_emit_eq!(mov_reg_addr(EAX, Address::sib(1, R12, RDI, 0)), 0x42, 0x8B, 0x04, 0x27);
        assert_emit_eq!(
            mov_reg_addr(ESP, Address::sib(1, R8, R13, 0)),
            0x43, 0x8B, 0x64, 0x05, 0x00
        );
        assert_emit_eq!(mov_reg_addr(R15D, Address::sib(1, RBX, R12, 0)), 0x45, 0x8B, 0x3C, 0x1C);
    }

    #[test]
    fn test_push_reg64() {
        assert_emit_eq!(push_reg(RAX), 0x50);
        assert_emit_eq!(push_reg(RCX), 0x51);
        assert_emit_eq!(push_reg(RDX), 0x52);
        assert_emit_eq!(push_reg(RBX), 0x53);
        assert_emit_eq!(push_reg(RSP), 0x54);
        assert_emit_eq!(push_reg(RBP), 0x55);
        assert_emit_eq!(push_reg(RSI), 0x56);
        assert_emit_eq!(push_reg(RDI), 0x57);
        assert_emit_eq!(push_reg(R8), 0x41, 0x50);
        assert_emit_eq!(push_reg(R9), 0x41, 0x51);
        assert_emit_eq!(push_reg(R10), 0x41, 0x52);
        assert_emit_eq!(push_reg(R11), 0x41, 0x53);
        assert_emit_eq!(push_reg(R12), 0x41, 0x54);
        assert_emit_eq!(push_reg(R13), 0x41, 0x55);
        assert_emit_eq!(push_reg(R14), 0x41, 0x56);
        assert_emit_eq!(push_reg(R15), 0x41, 0x57);
    }

    #[test]
    fn test_push_addr() {
        assert_emit_eq!(push_addr(Address::disp(RAX, 0)), 0xff, 0x30);
        assert_emit_eq!(push_addr(Address::disp(RCX, 0)), 0xff, 0x31);
        assert_emit_eq!(push_addr(Address::disp(RDX, 0)), 0xff, 0x32);
        assert_emit_eq!(push_addr(Address::disp(RBX, 0)), 0xff, 0x33);
        assert_emit_eq!(push_addr(Address::disp(RSP, 0)), 0xff, 0x34, 0x24);
        assert_emit_eq!(push_addr(Address::disp(RBP, 0)), 0xff, 0x75, 0x00);
        assert_emit_eq!(push_addr(Address::disp(RSI, 0)), 0xff, 0x36);
        assert_emit_eq!(push_addr(Address::disp(RDI, 0)), 0xff, 0x37);
        assert_emit_eq!(push_addr(Address::disp(R8, 0)), 0x41, 0xff, 0x30);
        assert_emit_eq!(push_addr(Address::disp(R9, 0)), 0x41, 0xff, 0x31);
        assert_emit_eq!(push_addr(Address::disp(R10, 0)), 0x41, 0xff, 0x32);
        assert_emit_eq!(push_addr(Address::disp(R11, 0)), 0x41, 0xff, 0x33);
        assert_emit_eq!(push_addr(Address::disp(R12, 0)), 0x41, 0xff, 0x34, 0x24);
        assert_emit_eq!(push_addr(Address::disp(R13, 0)), 0x41, 0xff, 0x75, 0x00);
        assert_emit_eq!(push_addr(Address::disp(R14, 0)), 0x41, 0xff, 0x36);
        assert_emit_eq!(push_addr(Address::disp(R15, 0)), 0x41, 0xff, 0x37);
    }

    #[test]
    fn test_push_addr_disp8() {
        assert_emit_eq!(push_addr(Address::disp(RAX, -39)), 0xFF, 0x70, 0xD9);
        assert_emit_eq!(push_addr(Address::disp(RBP, 88)), 0xFF, 0x75, 0x58);
        assert_emit_eq!(push_addr(Address::disp(RSP, 99)), 0xFF, 0x74, 0x24, 0x63);
        assert_emit_eq!(push_addr(Address::disp(R12, -13)), 0x41, 0xFF, 0x74, 0x24, 0xF3);
        assert_emit_eq!(push_addr(Address::disp(R13, 109)), 0x41, 0xFF, 0x75, 0x6D);
        assert_emit_eq!(push_addr(Address::disp(R15, 2)), 0x41, 0xFF, 0x77, 0x02);
    }

    #[test]
    fn test_pop_reg64() {
        assert_emit_eq!(pop_reg(RAX), 0x58);
        assert_emit_eq!(pop_reg(RCX), 0x59);
        assert_emit_eq!(pop_reg(RDX), 0x5a);
        assert_emit_eq!(pop_reg(RBX), 0x5b);
        assert_emit_eq!(pop_reg(RSP), 0x5c);
        assert_emit_eq!(pop_reg(RBP), 0x5d);
        assert_emit_eq!(pop_reg(RSI), 0x5e);
        assert_emit_eq!(pop_reg(RDI), 0x5f);
        assert_emit_eq!(pop_reg(R8), 0x41, 0x58);
        assert_emit_eq!(pop_reg(R9), 0x41, 0x59);
        assert_emit_eq!(pop_reg(R10), 0x41, 0x5a);
        assert_emit_eq!(pop_reg(R11), 0x41, 0x5b);
        assert_emit_eq!(pop_reg(R12), 0x41, 0x5c);
        assert_emit_eq!(pop_reg(R13), 0x41, 0x5d);
        assert_emit_eq!(pop_reg(R14), 0x41, 0x5e);
        assert_emit_eq!(pop_reg(R15), 0x41, 0x5f);
    }

    #[test]
    fn test_pop_addr() {
        assert_emit_eq!(pop_addr(Address::disp(RAX, 0)), 0x8f, 0x00);
        assert_emit_eq!(pop_addr(Address::disp(RCX, 0)), 0x8f, 0x01);
        assert_emit_eq!(pop_addr(Address::disp(RDX, 0)), 0x8f, 0x02);
        assert_emit_eq!(pop_addr(Address::disp(RBX, 0)), 0x8f, 0x03);
        assert_emit_eq!(pop_addr(Address::disp(RSP, 0)), 0x8f, 0x04, 0x24);
        assert_emit_eq!(pop_addr(Address::disp(RBP, 0)), 0x8f, 0x45, 0x00);
        assert_emit_eq!(pop_addr(Address::disp(RSI, 0)), 0x8f, 0x06);
        assert_emit_eq!(pop_addr(Address::disp(RDI, 0)), 0x8f, 0x07);
        assert_emit_eq!(pop_addr(Address::disp(R8, 0)), 0x41, 0x8f, 0x00);
        assert_emit_eq!(pop_addr(Address::disp(R9, 0)), 0x41, 0x8f, 0x01);
        assert_emit_eq!(pop_addr(Address::disp(R10, 0)), 0x41, 0x8f, 0x02);
        assert_emit_eq!(pop_addr(Address::disp(R11, 0)), 0x41, 0x8f, 0x03);
        assert_emit_eq!(pop_addr(Address::disp(R12, 0)), 0x41, 0x8f, 0x04, 0x24);
        assert_emit_eq!(pop_addr(Address::disp(R13, 0)), 0x41, 0x8f, 0x45, 0x00);
        assert_emit_eq!(pop_addr(Address::disp(R14, 0)), 0x41, 0x8f, 0x06);
        assert_emit_eq!(pop_addr(Address::disp(R15, 0)), 0x41, 0x8f, 0x07);
    }

    #[test]
    fn test_pop_ptr64_disp8() {
        assert_emit_eq!(pop_addr(Address::disp(RDX, -39)), 0x8F, 0x42, 0xD9);
        assert_emit_eq!(pop_addr(Address::disp(RBP, 88)), 0x8F, 0x45, 0x58);
        assert_emit_eq!(pop_addr(Address::disp(RSP, 99)), 0x8F, 0x44, 0x24, 0x63);
        assert_emit_eq!(pop_addr(Address::disp(R12, -13)), 0x41, 0x8F, 0x44, 0x24, 0xF3);
        assert_emit_eq!(pop_addr(Address::disp(R13, 109)), 0x41, 0x8F, 0x45, 0x6D);
        assert_emit_eq!(pop_addr(Address::disp(R8, 2)), 0x41, 0x8F, 0x40, 0x02);
    }

    #[test]
    fn test_sub_reg64_imm32() {
        assert_emit_eq!(sub_reg_imm32(RBP, -329), 0x48, 0x81, 0xED, 0xB7, 0xFE, 0xFF, 0xFF);
        assert_emit_eq!(sub_reg_imm32(RSP, 999), 0x48, 0x81, 0xEC, 0xE7, 0x03, 0x00, 0x00);
    }
}
