use ndsjit::disasm::armv4t::{
    AddrIndex, AddrMode, AddrOffset, Address, Cond, ExtraOperand, ImmShift, Instruction, Op,
    Operand, RegShift, Register, Shift, ShiftOp,
};

use std::error::Error;
use std::fmt::Display;
use std::slice::SliceIndex;
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug)]
pub struct AsmLine {
    pub line_no: usize,
    pub addr: u32,
    pub encoding: u32,
    pub instr: Instruction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    FormatError,
    FieldError { field: String, value: String },
}

impl ParseError {
    fn for_field(f: &str, v: &str) -> Self {
        Self::FieldError { field: String::from(f), value: String::from(v) }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

fn parse_dec<T: FromStr>(name: &str, val: &str) -> Result<T, ParseError> {
    val.parse().map_err(|_| ParseError::for_field(name, val))
}

fn parse_hex(name: &str, val: &str) -> Result<u32, ParseError> {
    u32::from_str_radix(val, 16).map_err(|_| ParseError::for_field(name, val))
}

fn parse_str_range<T, S>(name: &str, val: &str, range: S) -> Result<T, ParseError>
where
    T: FromStr,
    S: SliceIndex<str, Output = str>,
{
    val.get(range)
        .map(|s| s.trim().parse().ok())
        .flatten()
        .ok_or(ParseError::for_field(name, val))
}

fn parse_operand(input: &str) -> Result<Operand, ParseError> {
    let input = input.trim();
    let parse_err = || ParseError::for_field("operand", input);
    if let Ok(reg) = Register::from_str(input) {
        Ok(Operand::Reg(reg))
    } else if input.starts_with("#") {
        let rest = input.get(1..).ok_or(parse_err())?;
        let i = parse_dec("operand", rest)?;
        Ok(Operand::Imm(i))
    } else {
        Err(parse_err())
    }
}

fn parse_shift(input: &str) -> Result<Shift, ParseError> {
    let input = input.trim();
    let parse_err = ParseError::for_field("shift", input);
    let op: ShiftOp = parse_str_range("shift", input, 0..3)?;
    if let ShiftOp::RRX = op {
        if input.len() > 3 {
            return Err(parse_err);
        } else {
            return Ok(Shift::Imm(ImmShift { imm: 1, op }));
        }
    }
    let by = parse_operand(input.get(3..).ok_or(parse_err)?)?;
    match by {
        Operand::Imm(imm) => Ok(Shift::Imm(ImmShift { op, imm })),
        Operand::Reg(reg) => Ok(Shift::Reg(RegShift { op, reg })),
        _ => Err(ParseError::for_field("shift", input)),
    }
}

fn parse_set_flags(c: &str) -> Result<bool, ParseError> {
    (c.to_uppercase() == "S")
        .then_some(true)
        .ok_or(ParseError::for_field("S", c))
}

fn parse_mnemonic(input: &str) -> Result<(Op, Cond, bool), ParseError> {
    let err = ParseError::for_field("op", input);
    if input.len() < 1 {
        return Err(err.clone());
    }
    // Find the longest substring that converts to an Op
    let op = (1..=input.len())
        .rev()
        .find_map(|i| parse_str_range("op", input, 0..i).ok());
    let op: Op = op.ok_or(err.clone())?;
    let s = op.to_string().len();
    match input.len() - s {
        3 => Ok((
            op,
            parse_str_range("cond", input, s..s + 2)?,
            parse_set_flags(&input[s + 2..s + 3])?,
        )),
        2 => Ok((op, parse_str_range("cond", input, s..s + 2)?, false)),
        1 => Ok((op, Cond::AL, parse_set_flags(&input[s..s + 1])?)),
        0 => Ok((op, Cond::AL, false)),
        _ => Err(err.clone()),
    }
    // MLS - test this, MRS,
}

fn parse_address(input: &str) -> Result<(Address, Option<AddrOffset>), ParseError> {
    let input = input.trim();
    let err = ParseError::for_field("address", input);
    if !input.starts_with("[") {
        return Err(err.clone());
    }

    let interior: String = input[1..].chars().take_while(|c| *c != ']').collect();
    // + 2 because of []
    let rest = &input[2 + interior.len()..];

    let mut split = interior.split(',');

    let base: Register = parse_str_range("base", split.next().ok_or(err.clone())?, ..)?;
    let mut offset = if let Some(s) = split.next() {
        match parse_operand(s)? {
            Operand::Reg(reg) => {
                let shift = match split.next() {
                    Some(s) => match parse_shift(s)? {
                        Shift::Imm(imm) => Some(imm),
                        _ => {
                            return Err(ParseError::for_field("offset shift", s));
                        }
                    },
                    None => None,
                };

                Some(AddrOffset::Index(AddrIndex { reg, shift }))
            }
            Operand::Imm(imm) => Some(AddrOffset::Imm(imm as i32)),
            _ => {
                return Err(ParseError::for_field("offset", s));
            }
        }
    } else {
        None
    };
    // Should have used up all of the text between []
    if split.next().is_some() {
        return Err(err.clone());
    }

    let mode = match rest.trim() {
        "" => AddrMode::Offset,
        "!" => AddrMode::PreIndex,
        s => {
            if offset.is_some() || !s.starts_with(",") {
                return Err(err.clone());
            }
            // Left with either , Rs, SHFT #IMM or , #IMM
            let mut split = s[1..].split(",");
            match parse_operand(split.next().ok_or(err.clone())?)? {
                Operand::Imm(imm) => {
                    offset = Some(AddrOffset::Imm(imm as i32));
                    AddrMode::PostIndex
                }
                Operand::Reg(reg) => {
                    let shift = match split.next() {
                        Some(s) => match parse_shift(s)? {
                            Shift::Imm(imm) => Some(imm),
                            _ => {
                                return Err(ParseError::for_field("offset shift", s));
                            }
                        },
                        None => None,
                    };
                    offset = Some(AddrOffset::Index(AddrIndex { reg, shift }));
                    AddrMode::PostIndex
                }
                _ => {
                    return Err(err.clone());
                }
            }
        }
    };

    Ok((Address { base, mode }, offset))
}

/// Parse a line of output from gnu-as
impl FromStr for AsmLine {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Assembler output includes a lot of empty strings, and page headers
        if input.len() == 0 || input.trim().starts_with("ARM GAS") {
            return Err(ParseError::FormatError);
        }

        let input = input.replace(".syntax unified; ", "");

        let mut split = input.trim().split_whitespace();
        let mut next_split = || split.next().ok_or(ParseError::FormatError);

        let ind: usize = parse_dec("index", next_split()?)?;
        let addr = parse_hex("address", next_split()?)?;
        let encoding = parse_hex("encoding", next_split()?)?;
        let (op, cond, s) = parse_mnemonic(next_split()?)?;

        let mut instr = Instruction { cond, op, set_flags: s, ..Default::default() };

        let rest = split.collect::<Vec<&str>>().join(" ");
        for (i, s) in rest.split(",").enumerate() {
            match parse_operand(s) {
                Ok(operand) => {
                    instr.operands.push(operand);
                }
                Err(_) => {
                    instr.extra = Some(ExtraOperand::Shift(parse_shift(s)?));
                }
            }
        }d

        // These instructions always set flags and do not support the S suffix
        if [Op::TEQ, Op::TST, Op::CMN, Op::CMP].contains(&instr.op) {
            instr.set_flags = true;
        }

        Ok(AsmLine { line_no: ind, addr, encoding, instr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndsjit::disasm::armv4t::{
        AddrIndex, AddrMode::*, AddrOffset, Address, Op, Register::*, Shift, ShiftOp::*,
    };

    #[test]
    fn test_parse_operand() -> Result<(), ParseError> {
        assert_eq!(parse_operand("sp")?, Operand::Reg(SP));
        assert_eq!(parse_operand("#123494")?, Operand::Imm(123494));
        assert!(parse_operand("LSL #1234").is_err());
        Ok(())
    }

    #[test]
    fn test_asm_line_from_str() -> Result<(), ParseError> {
        let line = String::from("   2 0004 A00ED614      ANDGE sp, lr, r4,LSL r6\n");
        let asm: AsmLine = line.parse()?;
        assert_eq!(
            asm,
            AsmLine {
                line_no: 2,
                addr: 4,
                encoding: 0xa00ed614,
                instr: Instruction {
                    cond: Cond::GE,
                    op: Op::AND,
                    operands: vec![Operand::Reg(SP), Operand::Reg(LR), Operand::Reg(R4)],
                    set_flags: false,
                    extra: Some(RegShift { op: LSL, reg: R6 }.into())
                }
            }
        );
        Ok(())
    }

    #[test]
    fn test_empty_line() {
        assert_eq!(AsmLine::from_str("").unwrap_err(), ParseError::FormatError);
    }

    #[test]
    fn test_header_line() {
        let line = "ARM GAS                          page 1";
        assert_eq!(AsmLine::from_str(line).unwrap_err(), ParseError::FormatError);
        let line = "\x0cARM GAS                          page 37";
        assert_eq!(AsmLine::from_str(line).unwrap_err(), ParseError::FormatError);
    }

    #[test]
    fn test_parse_shift() -> Result<(), ParseError> {
        assert_eq!(parse_shift("LSL#23")?, Shift::Imm(ImmShift { op: LSL, imm: 23 }));
        assert_eq!(parse_shift("RRX")?, Shift::Imm(ImmShift { op: RRX, imm: 1 }));
        Ok(())
    }

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(parse_mnemonic("MLS").unwrap(), (Op::MLS, Cond::AL, false));
        assert_eq!(parse_mnemonic("MLSLS").unwrap(), (Op::MLS, Cond::LS, false));
        assert_eq!(parse_mnemonic("MLSLSS").unwrap(), (Op::MLS, Cond::LS, true));
        assert_eq!(parse_mnemonic("MLSS").unwrap(), (Op::MLS, Cond::AL, true));
    }

    #[test]
    fn test_parse_address() {
        let res = assert_eq!(
            parse_address("[r9, r1]!").unwrap(),
            (Address { base: R9, mode: PreIndex }, Some(AddrIndex { reg: R1, shift: None }.into()))
        );

        assert_eq!(
            parse_address("[ r0, r12 ]").unwrap(),
            (Address { base: R0, mode: Offset }, Some(AddrIndex { reg: R12, shift: None }.into()))
        );
        assert_eq!(
            parse_address("[ r8, #101 ]!").unwrap(),
            (Address { base: R8, mode: PreIndex }, Some(AddrOffset::Imm(101)))
        );
        assert_eq!(
            parse_address("[ r8, #93 ]").unwrap(),
            (Address { base: R8, mode: Offset }, Some(AddrOffset::Imm(93)))
        );
        assert_eq!(
            parse_address("[ r9 ], r5 ").unwrap(),
            (
                Address { base: R9, mode: PostIndex },
                Some(AddrIndex { reg: R5, shift: None }.into())
            )
        );
        assert_eq!(
            parse_address("[ r0 ], #123 ").unwrap(),
            (Address { base: R0, mode: PostIndex }, Some(AddrOffset::Imm(123)))
        );
        assert_eq!(parse_address("[ r1 ]").unwrap(), (Address { base: R1, mode: Offset }, None));
        assert_eq!(
            parse_address("[ r1, r2, LSL #12 ]").unwrap(),
            (
                Address { base: R1, mode: Offset },
                Some(AddrIndex { reg: R2, shift: Some(ImmShift { op: LSL, imm: 12 }) }.into())
            )
        );
        assert_eq!(
            parse_address("[ r12, sp, ROR #31 ]!").unwrap(),
            (
                Address { base: R12, mode: PreIndex },
                Some(AddrIndex { reg: SP, shift: Some(ImmShift { op: ROR, imm: 31 }) }.into()),
            )
        );
        assert_eq!(
            parse_address("[r12], #702").unwrap(),
            (Address { base: R12, mode: PostIndex }, Some(AddrOffset::Imm(702)),)
        );
        assert_eq!(
            parse_address("[r12], R3, ASR #2").unwrap(),
            (
                Address { base: R12, mode: PostIndex },
                Some(AddrIndex { reg: R3, shift: Some(ImmShift { op: ASR, imm: 2 }) }.into())
            )
        );
        assert_eq!(
            parse_address("[r2], LR").unwrap(),
            (
                Address { base: R2, mode: PostIndex },
                Some(AddrIndex { reg: LR, shift: None }.into())
            )
        );
    }

    #[test]
    fn test_split_behaviour() {
        dbg!("abc".split(',').next().unwrap());
    }
}
