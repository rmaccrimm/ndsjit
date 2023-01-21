use super::DisasmError;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Write;
use std::str::FromStr;
use std::string::ParseError;
use strum::EnumString;

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumString)]
pub enum Cond {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
}

impl TryFrom<u32> for Cond {
    type Error = DisasmError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let cond = match value {
            0 => Cond::EQ,
            1 => Cond::NE,
            2 => Cond::CS,
            3 => Cond::CC,
            4 => Cond::MI,
            5 => Cond::PL,
            6 => Cond::VS,
            7 => Cond::VC,
            8 => Cond::HI,
            9 => Cond::LS,
            10 => Cond::GE,
            11 => Cond::LT,
            12 => Cond::GT,
            13 => Cond::LE,
            14 => Cond::AL,
            _ => return Err(DisasmError::new("invalid cond value", value)),
        };
        Ok(cond)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    SP,
    LR,
    PC,
    #[strum(disabled)]
    FLAGS,
}

impl TryFrom<u32> for Register {
    type Error = DisasmError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let reg = match value {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,
            8 => Register::R8,
            9 => Register::R9,
            10 => Register::R10,
            11 => Register::R11,
            12 => Register::R12,
            13 => Register::SP,
            14 => Register::LR,
            15 => Register::PC,
            _ => return Err(DisasmError::new("invalid register value", value)),
        };
        Ok(reg)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumString)]
pub enum ShiftOp {
    LSL,
    LSR,
    ASR,
    ROR,
    RRX,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShiftType {
    Reg(Register),
    Imm(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Shift {
    pub shift_type: ShiftType,
    pub op: ShiftOp,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AddrMode {
    Offset,
    PreIndex,
    PostIndex,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Address {
    pub base: Register,
    pub mode: AddrMode,
    pub write_back: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Reg(Register),
    Imm(u32),
    Addr(Address),
}

/// Helper methods for filling out operand lists
impl Operand {
    pub fn register(reg: Register) -> Option<Self> {
        Some(Self::Reg(reg))
    }

    pub fn immediate(imm: u32) -> Option<Self> {
        Some(Self::Imm(imm))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumString)]
pub enum Op {
    UNDEFINED,
    ADC,
    ADD,
    ADDW,
    ADR, // Encoded as ADD/SUB PC
    AND,
    ASR,
    B,
    BFC,
    BFI,
    BIC,
    BKPT,
    BL,
    BLX,
    BX,
    BXJ,
    CBNZ,
    CBZ,
    CDP,
    CDP2,
    CLREX,
    CLZ,
    CMN,
    CMP,
    CPS,
    CPSID,
    CPSIE,
    DBG,
    DMB,
    DSB,
    ENTERX,
    EOR,
    EORS,
    ERET,
    FLDMDBX,
    FLDMIAX,
    FSTMDBX,
    FSTMIAX,
    FSTMX,
    HINT,
    HVC,
    ISB,
    IT,
    LDA,
    LDAB,
    LDAH,
    LDAEX,  // A32
    LDAEXB, // A32
    LDAEXH, // A32
    LDAEXD, // A32
    LDC,
    LDC2,
    LDC2L,
    LDCL,
    LDM,
    LDMDA,
    LDMDB,
    LDMIA,
    LDMIB,
    LDR,
    LDRB,
    LDRBT,
    LDRD,
    LDREX,
    LDREXB,
    LDREXD,
    LDREXH,
    LDRH,
    LDRHT,
    LDRSB,
    LDRSBT,
    LDRSH,
    LDRSHT,
    LDRT,
    LEAVEX,
    LSL,
    LSR,
    MCR,
    MCR2,
    MCRR,
    MCRR2,
    MLA,
    MLS,
    MOV,
    MOVT,
    MOVW,
    MRC,
    MRC2,
    MRRC,
    MRRC2,
    MRS,
    MSR,
    MUL,
    MVN,
    NOP,
    ORN,
    ORR,
    PKHBT,
    PKHTB,
    PLD,
    PLDW,
    PLI,
    POP,
    PUSH,
    QADD,
    QADD16,
    QADD8,
    QASX,
    QDADD,
    QDSUB,
    QSAX,
    QSUB,
    QSUB16,
    QSUB8,
    RBIT,
    REV,
    REV16,
    REVSH,
    RFE,
    RFEDA,
    RFEDB,
    RFEIA,
    RFEIB,
    ROR,
    RRX,
    RSB,
    RSC,
    SADD16,
    SADD8,
    SASX,
    SBC,
    SBCS,
    SBFX,
    SDIV,
    SEL,
    SETEND,
    SEV,
    SHADD16,
    SHADD8,
    SHASX,
    SHSAX,
    SHSUB16,
    SHSUB8,
    SMC,
    SMLABB,
    SMLABT,
    SMLAD,
    SMLADX,
    SMLAL,
    SMLALBB,
    SMLALBT,
    SMLALD,
    SMLALDX,
    SMLALTB,
    SMLALTT,
    SMLATB,
    SMLATT,
    SMLAWB,
    SMLAWT,
    SMLSD,
    SMLSDX,
    SMLSLD,
    SMLSLDX,
    SMMLA,
    SMMLAR,
    SMMLS,
    SMMLSR,
    SMMUL,
    SMMULR,
    SMUAD,
    SMUADX,
    SMULBB,
    SMULBT,
    SMULL,
    SMULTB,
    SMULTT,
    SMULWB,
    SMULWT,
    SMUSD,
    SMUSDT,
    SMUSDX,
    SRS,
    SRSDA,
    SRSDB,
    SRSIA,
    SRSIB,
    SSAT,
    SSAT16,
    SSAX,
    SSUB16,
    SSUB8,
    STC,
    STC2,
    STC2L,
    STCL,
    STL, // A32
    STLB,
    STLH,
    STLEX,  // A32
    STLEXB, // A32
    STLEXH, // A32
    STLEXD, // A32
    STM,
    STMBD,
    STMDA,
    STMDB,
    STMIA,
    STMIB,
    STR,
    STRB,
    STRBT,
    STRD,
    STREX,
    STREXB,
    STREXD,
    STREXH,
    STRH,
    STRHT,
    STRT,
    SUB,
    SUBW,
    SVC,
    SWP,
    SWPB,
    SXTAB,
    SXTAB16,
    SXTAH,
    SXTB,
    SXTB16,
    SXTH,
    TBB,
    TBH,
    TEQ,
    TRAP,
    TRT,
    TST,
    UADD16,
    UADD8,
    UASX,
    UBFX,
    UDF,
    UDIV,
    UHADD16,
    UHADD8,
    UHASX,
    UHSAX,
    UHSUB16,
    UHSUB8,
    UMAAL,
    UMLAL,
    UMULL,
    UQADD16,
    UQADD8,
    UQASX,
    UQSAX,
    UQSUB16,
    UQSUB8,
    USAD8,
    USADA8,
    USAT,
    USAT16,
    USAX,
    USUB16,
    USUB8,
    UXTAB,
    UXTAB16,
    UXTAH,
    UXTB,
    UXTB16,
    UXTH,
    VABA,
    VABAL,
    VABD,
    VABDL,
    VABS,
    VACGE,
    VACGT,
    VADD,
    VADDHN,
    VADDL,
    VADDW,
    VAND,
    VBIC,
    VBIF,
    VBIT,
    VBSL,
    VCEQ,
    VCGE,
    VCGT,
    VCLE,
    VCLS,
    VCLT,
    VCLZ,
    VCMP,
    VCMPE,
    VCNT,
    VCVT,
    VCVTA,
    VCVTB,
    VCVTM,
    VCVTN,
    VCVTP,
    VCVTR,
    VCVTT,
    VDIV,
    VDUP,
    VEOR,
    VEXT,
    VFMA,
    VFMS,
    VFNMA,
    VFNMS,
    VHADD,
    VHSUB,
    VLD1,
    VLD2,
    VLD3,
    VLD4,
    VLDM,
    VLDMDB,
    VLDMIA,
    VLDR,
    VMAX,
    VMAXNM,
    VMIN,
    VMINM,
    VMLA,
    VMLAL,
    VMLS,
    VMLSL,
    VMOV,
    VMOVL,
    VMOVN,
    VMRS,
    VMSR,
    VMUL,
    VMULL,
    VMVN,
    VNEG,
    VNMLA,
    VNMLS,
    VNMUL,
    VORN,
    VORR,
    VPADAL,
    VPADD,
    VPADDL,
    VPMAX,
    VPMIN,
    VPOP,
    VPUSH,
    VQABS,
    VQADD,
    VQDMLAL,
    VQDMLSL,
    VQDMULH,
    VQDMULL,
    VQMOVN,
    VQMOVUN,
    VQNEG,
    VQRDMULH,
    VQRSHL,
    VQRSHRN,
    VQRSHRUN,
    VQSHL,
    VQSHLU,
    VQSHRN,
    VQSHRUN,
    VQSUB,
    VRADDHN,
    VRECPE,
    VRECPS,
    VREV16,
    VREV32,
    VREV64,
    VRHADD,
    VRHSUB,
    VRINTA,
    VRINTM,
    VRINTN,
    VRINTP,
    VRINTR,
    VRINTX,
    VRINTZ,
    VRSHL,
    VRSHR,
    VRSHRN,
    VRSQRTE,
    VRSQRTS,
    VRSRA,
    VRSUBHN,
    VSEL,
    VSHL,
    VSHLL,
    VSHR,
    VSHRN,
    VSLI,
    VSQRT,
    VSRA,
    VSRI,
    VST1,
    VST2,
    VST3,
    VST4,
    VSTM,
    VSTMDB,
    VSTMIA,
    VSTR,
    VSUB,
    VSUBHN,
    VSUBL,
    VSUBW,
    VSWP,
    VTBL,
    VTBX,
    VTRN,
    VTST,
    VUZP,
    VZIP,
    WFE,
    WFI,
    YIELD,
}

const MAX_NUM_OPERANDS: usize = 4;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub cond: Cond,
    pub op: Op,
    pub operands: [Option<Operand>; MAX_NUM_OPERANDS],
    // will need to move back to Operand if it's ever possible to have more than 1 per op
    pub shift: Option<Shift>,
    pub set_flags: bool,
}

impl Default for Instruction {
    fn default() -> Self {
        Self {
            cond: Cond::AL,
            op: Op::NOP,
            operands: [None; 4],
            shift: None,
            set_flags: false,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = if self.set_flags { "S" } else { "" };
        let mut operand_str = String::new();
        for i in 0..MAX_NUM_OPERANDS {
            match self.operands[i] {
                None => {
                    break;
                }
                Some(Operand::Reg(reg)) => {
                    if i != 0 {
                        operand_str.push_str(", ");
                    }
                    write!(operand_str, "{:?}", reg)?;
                }
                Some(Operand::Imm(imm)) => {
                    write!(operand_str, ", #{}", imm)?;
                }
                _ => todo!(),
            }
        }
        if let Some(shift) = self.shift {
            match shift.shift_type {
                ShiftType::Reg(reg) => {
                    write!(operand_str, ", {:?} {:?}", shift.op, reg)?;
                }
                ShiftType::Imm(imm) => {
                    write!(operand_str, ", {:?} {}", shift.op, imm)?;
                }
            }
        }
        write!(
            f,
            "{op:?}{cond:?}{s} {operands}",
            op = self.op,
            cond = self.cond,
            s = s,
            operands = operand_str
        )
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use std::str::FromStr;

    use super::{
        Cond::*,
        Instruction,
        Op::{self, *},
        Operand,
        Register::*,
    };

    #[test]
    fn test_instr_display() {
        let instr = Instruction {
            cond: EQ,
            op: AND,
            operands: [
                Operand::register(R12),
                Operand::register(PC),
                Operand::immediate(12),
                None,
            ],
            ..Default::default()
        };
        assert_eq!(instr.to_string(), "ANDEQ R12, PC, #12");
    }

    #[test]
    fn test_enum_str_derive() {
        assert_eq!(Op::from_str("AND").unwrap(), AND);
    }
}
