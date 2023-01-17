use ndsjit::disasm::armv4t::{
    Cond, ImmValue, Instruction, Op, Operand, Register, Shift, ShiftType,
};
use std::error::Error;
use std::fmt::Display;
use std::ops::Range;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AsmLine {
    pub line_no: usize,
    pub addr: u32,
    pub encoding: u32,
    pub instr: Instruction,
}

#[derive(Debug, PartialEq, Eq)]
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
    val.get(i)
        .map(|s| s.parse().ok())
        .flatten()
        .ok_or(ParseError::for_field(name, val))
}

fn parse_operand_no_shift(input: &str) -> Result<Operand, ParseError> {
    let input = input.trim();
    let parse_err = || ParseError::for_field("operand", input);
    if let Ok(reg) = Register::from_str(input) {
        Ok(Operand::Reg { reg, shift: None })
    } else if input.starts_with("#") {
        let rest = input.get(1..).ok_or(parse_err())?;
        let i = parse_dec("operand", rest)?;
        Ok(Operand::unsigned(i).unwrap())
    } else {
        Err(parse_err())
    }
}

fn parse_shift(input: &str) -> Result<Shift, ParseError> {
    let parse_err = ParseError::for_field("shift", input);
    let kind: ShiftType = parse_str_range("shift", input, 0..3)?;
    if let ShiftType::RRX = kind {
        if input.len() > 3 {
            return Err(parse_err);
        } else {
            return Ok(Shift::ImmShift {
                shift_type: kind,
                shift_amt: ImmValue::Unsigned(1),
            });
        }
    }
    let by = parse_operand_no_shift(input.get(3..).ok_or(parse_err)?)?;
    match by {
        Operand::Imm(imm) => Ok(Shift::ImmShift {
            shift_type: kind,
            shift_amt: imm,
        }),
        Operand::Reg { reg, shift: _ } => Ok(Shift::RegShift {
            shift_type: kind,
            shift_reg: reg,
        }),
    }
}

fn parse_operands(input: &str) -> Result<Vec<Operand>, ParseError> {
    let mut split = input.split(",");
    let parse_err = || ParseError::for_field("operands", input);

    let mut ops: Vec<Operand> = Vec::new();
    let mut curr = split.next().ok_or(parse_err())?.trim();
    let mut next = split.next();
    loop {
        let mut op = parse_operand_no_shift(curr)?;
        if let Operand::Reg { reg, shift: _ } = op {
            if let Some(s) = next {
                // Search next token for a shift
                if let Ok(shift) = parse_shift(s.trim()) {
                    op = Operand::Reg {
                        reg,
                        shift: Some(shift),
                    };
                    next = split.next();
                }
            }
        }
        ops.push(op);
        if next.is_none() {
            break;
        }
        curr = next.unwrap().trim();
        next = split.next();
    }
    Ok(ops)
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

        let rest = split.collect::<Vec<&str>>().join(" ");
        let operands = parse_operands(&rest)?;
        if operands.len() > 3 {
            return Err(ParseError::for_field("operands", &rest));
        }

        let instr = Instruction {
            cond,
            op,
            operands: [
                operands.get(0).cloned(),
                operands.get(1).cloned(),
                operands.get(2).cloned(),
            ],
            set_flags: s,
        };

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
    use ndsjit::disasm::armv4t::{ImmValue, Register::*, ShiftType::*};

    #[test]
    fn test_parse_operand_no_shift() -> Result<(), ParseError> {
        assert_eq!(parse_operand_no_shift("sp")?, Operand::unshifted(SP).unwrap());
        assert_eq!(parse_operand_no_shift("#123494")?, Operand::unsigned(123494).unwrap());
        assert!(parse_operand_no_shift("LSL #1234").is_err());
        Ok(())
    }

    #[test]
    fn test_parse_operands() -> Result<(), ParseError> {
        let res = parse_operands("lr, ROR #123,pc , LSL r1,r2, r3, #99")?;
        assert_eq!(
            res[0],
            Operand::shifted(
                LR,
                Shift::ImmShift {
                    shift_type: ROR,
                    shift_amt: ImmValue::Unsigned(123)
                }
            )
            .unwrap()
        );
        assert_eq!(
            res[1],
            Operand::shifted(
                PC,
                Shift::RegShift {
                    shift_type: LSL,
                    shift_reg: R1
                }
            )
            .unwrap()
        );
        assert_eq!(res[2], Operand::unshifted(R2).unwrap());
        assert_eq!(res[3], Operand::unshifted(R3).unwrap());
        assert_eq!(res[4], Operand::unsigned(99).unwrap());

        assert!(parse_operands("lqq, ROR #123, pc").is_err());
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
            Shift::ImmShift {
                shift_type: LSL,
                shift_amt: ImmValue::Unsigned(23)
            }
        );
        assert_eq!(
            parse_shift("RRX").unwrap(),
            Shift::ImmShift {
                shift_type: RRX,
                shift_amt: ImmValue::Unsigned(1)
            }
        );
        Ok(())
    }
}
