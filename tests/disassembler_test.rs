use ndsjit::disasm::armv4t::{Cond, Instruction, Op, Operand, Register, Shift, ShiftType};
use std::error::Error;
use std::fmt::Display;
use std::ops::{Range, RangeBounds};
use std::slice::SliceIndex;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct AsmLine {
    line_no: usize,
    addr: u32,
    encoding: u32,
    instr: Instruction,
}

#[derive(Debug)]
enum ParseError {
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
    val.get(i)
        .map(|s| s.parse().ok())
        .flatten()
        .ok_or(ParseError::for_field(name, val))
}

/// Parse a line of output from gnu-as
fn parse_asm_line(txt: String) -> Result<AsmLine, ParseError> {
    let split = txt.trim().split_whitespace();
    let next_split = || split.next().ok_or(ParseError::FormatError);

    let ind: usize = parse_dec("index", next_split()?)?;
    let addr = parse_hex("address", next_split()?)?;
    let encoding = parse_hex("encoding", next_split()?)?;
    let mnemonic = next_split()?;
    let op: Op = parse_str_range("op", mnemonic, 0..3)?;
    let cond: Cond = parse_str_range("cond", mnemonic, 3..5)?;

    let s = match mnemonic.get(5..6) {
        Some(s) => match s.to_uppercase() == "S" {
            true => Ok(true),
            false => Err(ParseError::for_field("S", mnemonic)),
        },
        None => Ok(false),
    }?;
    // .map(|s| {
    //     if s.to_uppercase() == "S" {
    //         Ok(true)
    //     } else {
    //         Err(ParseError::for_field("S", mnemonic))
    //     }
    // })
    // .transpose()?
    // .is_some();

    // let op: Op = mnemonic
    // .get(0..3)
    // .map(|s| s.parse().ok())
    // .flatten()
    // .ok_or(ParseError::for_field("op", mnemonic))?;
    // let cond = mnemonic.get(4..6).map_or(Ok(Cond::AL), |s| Cond::from_str(s))?;
    // let s = mnemonic.get(6).map_or(false, |s| )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asm_line() {
        let line = String::from("   2 0004 A00ED614      ANDGE sp, lr, r4, LSL r6\n");
        assert_eq!(
            parse_asm_line(line).unwrap(),
            AsmLine {
                line_no: 2,
                addr: 4,
                encoding: 0xa00ed614,
                instr: Instruction {
                    cond: Cond::GE,
                    op: Op::AND,
                    operands: [
                        Some(Operand::Reg {
                            reg: Register::SP,
                            shift: None
                        }),
                        Some(Operand::Reg {
                            reg: Register::LR,
                            shift: None
                        }),
                        Some(Operand::Reg {
                            reg: Register::R4,
                            shift: Some(Shift::RegShift {
                                shift_type: ShiftType::LSL,
                                shift_reg: Register::R6
                            })
                        })
                    ],
                    set_flags: false
                }
            }
        )
    }
}
