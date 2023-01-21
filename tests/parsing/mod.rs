use ndsjit::disasm::armv4t::{Cond, Instruction, Op, Operand, Register, Shift, ShiftOp, ShiftType};
use std::error::Error;
use std::fmt::Display;
use std::ops::Range;
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
        Self::FieldError {
            field: String::from(f),
            value: String::from(v),
        }
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

fn parse_str_range<T: FromStr>(name: &str, val: &str, i: Range<usize>) -> Result<T, ParseError> {
    let res = val
        .get(i)
        .map(|s| s.parse().ok())
        .flatten()
        .ok_or(ParseError::for_field(name, val));
    res
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
    let parse_err = ParseError::for_field("shift", input);
    let op: ShiftOp = parse_str_range("shift", input, 0..3)?;
    if let ShiftOp::RRX = op {
        if input.len() > 3 {
            return Err(parse_err);
        } else {
            return Ok(Shift {
                shift_type: ShiftType::Imm(1),
                op,
            });
        }
    }
    let by = parse_operand(input.get(3..).ok_or(parse_err)?)?;
    match by {
        Operand::Imm(imm) => Ok(Shift {
            op: op,
            shift_type: ShiftType::Imm(imm),
        }),
        Operand::Reg(reg) => Ok(Shift {
            op: op,
            shift_type: ShiftType::Reg(reg),
        }),
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

/// Parse a line of output from gnu-as
impl FromStr for AsmLine {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Assembler output includes a lot of empty strings, and page headers
        if input.len() == 0 || input.trim().starts_with("ARM GAS") {
            return Err(ParseError::FormatError);
        }

        let mut split = input.trim().split_whitespace();
        let mut next_split = || split.next().ok_or(ParseError::FormatError);

        let ind: usize = parse_dec("index", next_split()?)?;
        let addr = parse_hex("address", next_split()?)?;
        let encoding = parse_hex("encoding", next_split()?)?;
        let (op, cond, s) = parse_mnemonic(next_split()?)?;

        let mut instr = Instruction {
            cond,
            op,
            set_flags: s,
            ..Default::default()
        };

        let rest = split.collect::<Vec<&str>>().join(" ");
        for (i, s) in rest.split(",").enumerate() {
            match parse_operand(s) {
                Ok(operand) => {
                    instr.operands[i] = Some(operand);
                }
                Err(_) => {
                    instr.shift = Some(parse_shift(s)?);
                }
            }
        }

        // These instructions always set flags and do not support the S suffix
        if [Op::TEQ, Op::TST, Op::CMN, Op::CMP].contains(&instr.op) {
            instr.set_flags = true;
        }

        Ok(AsmLine {
            line_no: ind,
            addr,
            encoding,
            instr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndsjit::disasm::armv4t::{Op, Register::*, Shift, ShiftOp::*, ShiftType};

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
                    operands: [
                        Some(Operand::Reg(SP)),
                        Some(Operand::Reg(LR)),
                        Some(Operand::Reg(R4)),
                        None,
                    ],
                    set_flags: false,
                    shift: Some(Shift {
                        op: LSL,
                        shift_type: ShiftType::Reg(R6),
                    })
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
        assert_eq!(
            parse_shift("LSL#23")?,
            Shift {
                op: LSL,
                shift_type: ShiftType::Imm(23)
            }
        );
        assert_eq!(
            parse_shift("RRX")?,
            Shift {
                op: RRX,
                shift_type: ShiftType::Imm(1)
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(parse_mnemonic("MLS").unwrap(), (Op::MLS, Cond::AL, false));
        assert_eq!(parse_mnemonic("MLSLS").unwrap(), (Op::MLS, Cond::LS, false));
        assert_eq!(parse_mnemonic("MLSLSS").unwrap(), (Op::MLS, Cond::LS, true));
        assert_eq!(parse_mnemonic("MLSS").unwrap(), (Op::MLS, Cond::AL, true));
    }
}
