use ndsjit::disasm::armv4t::{
    parsing::{instruction, ParseResult},
    Cond, Instruction, Operand, RegShift,
};

use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use std::convert::From;

use nom::{
    self,
    bytes::complete::tag,
    character::complete::hex_digit1,
    character::complete::{multispace0, multispace1, u32 as match_u32},
    combinator::{map_res, opt},
    error::{convert_error, VerboseError},
};

#[derive(PartialEq, Eq, Debug)]
pub struct AsmLine {
    pub line_no: usize,
    pub addr: u32,
    pub encoding: u32,
    pub instr: Instruction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    // Extra input lines that we don't attempt to parse
    NotParsed,
    // A line for which parsing was attempted, but failed
    Failure { msg: String },
}

impl ParseError {
    fn from_nom_err(input: &str, err: nom::Err<VerboseError<&str>>) -> Self {
        match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                ParseError::Failure { msg: convert_error(input, e) }
            }
            _ => ParseError::Failure { msg: format!("{:?}", err) },
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NotParsed => {
                write!(f, "{:?}", self)
            }
            ParseError::Failure { msg } => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl Error for ParseError {}

fn asm_line(i: &str) -> ParseResult<AsmLine> {
    let mut hex = map_res(hex_digit1, |s| u32::from_str_radix(s, 16));
    let (i, _) = multispace0(i)?;
    let (i, ind) = match_u32(i)?;
    let (i, _) = multispace1(i)?;
    let (i, addr) = hex(i)?;
    let (i, _) = multispace1(i)?;
    let (i, encoding) = hex(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = opt(tag(".syntax unified;"))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, instr) = instruction(i)?;
    Ok((i, AsmLine { line_no: ind as usize, addr, encoding, instr }))
}

/// Parse a line of output from gnu-as
impl FromStr for AsmLine {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Assembler output includes a lot of empty strings, and page headers
        if input.len() == 0 || input.trim().starts_with("ARM GAS") {
            return Err(ParseError::NotParsed);
        }
        let (_, res) = asm_line(input).map_err(|e| ParseError::from_nom_err(input, e))?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndsjit::disasm::armv4t::{Op, Register::*, ShiftOp::*};

    #[test]
    fn test_asm_line_from_str() {
        let line = String::from("   2 0004 A00ED614      ANDGE SP, lr, r4,LSL r6\n");
        let asm: Result<AsmLine, ParseError> = line.parse();
        if let Err(ParseError::Failure { msg }) = &asm {
            println!("{}", msg);
        }
        assert_eq!(
            asm,
            Ok(AsmLine {
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
            })
        );
    }
}
