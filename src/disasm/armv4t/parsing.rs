use std::str::FromStr;

use crate::disasm::armv4t::AddrIndex;

use super::{AddrMode, AddrOffset, Address, Cond, ImmShift, Op, Register, ShiftOp};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{
        alpha1, alphanumeric1, char as match_char, multispace0, multispace1, one_of,
        u32 as match_u32,
    },
    combinator::{map, map_res, opt},
    error::{context, convert_error, ErrorKind, ParseError},
    Err, IResult, Needed,
};
// use strum::ParseError;

fn op<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Op, E> {
    map_res(alpha1, |i| Op::from_str(i))(input)
}

fn cond(input: &str) -> IResult<&str, Cond> {
    map_res(take(2usize), Cond::from_str)(input)
}

fn register(input: &str) -> IResult<&str, Register> {
    map_res(alphanumeric1, Register::from_str)(input)
}

fn imm_val(i: &str) -> IResult<&str, u32> {
    let (i, _) = match_char('#')(i)?;
    let (i, val) = match_u32(i)?;
    Ok((i, val))
}

fn mnemonic(input: &str) -> IResult<&str, (Op, Cond, bool)> {
    if input.len() < 1 {
        return Err(Err::Incomplete(Needed::new(1)));
    }
    // Try to parse an op repeatedly, starting with full input and decreasing in length
    let mut i = input.len();
    let (rest, op) = loop {
        let result = context("op", op)(&input[0..i]);
        match result {
            Ok((_, op)) => {
                break Ok((&input[i..], op));
            }
            Err(_) => {
                if i == 0 {
                    break result;
                }
            }
        }
        i -= 1;
    }?;
    let (rest, cond) = opt(cond)(rest).unwrap();
    let cond = cond.unwrap_or(Cond::AL);

    let parse_s = one_of::<_, _, (&str, ErrorKind)>("sS");
    let (rest, s) = opt(parse_s)(rest).unwrap();

    Ok((rest, (op, cond, s.is_some())))
}

/// Parses an immediate shift, starting from the comma following an index register
/// e.g. [r0, r1, lsl #123]!
///             ^--------^ parses this span
fn imm_shift(i: &str) -> IResult<&str, ImmShift> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, op) = map_res(take(3usize), ShiftOp::from_str)(i)?;
    let (i, _) = multispace1(i)?;
    let (i, imm) = imm_val(i)?;
    Ok((i, ImmShift { op, imm }))
}

/// Parse an index register offset with optional shift
fn index_offset(i: &str) -> IResult<&str, AddrOffset> {
    let (i, reg) = register(i)?;
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, shift) = opt(imm_shift)(i)?;
    Ok((i, AddrIndex { reg, shift }.into()))
}

// Parse an immediate address offset value (signed 32-bit)
fn imm_offset(i: &str) -> IResult<&str, AddrOffset> {
    let (i, imm) = imm_val(i)?;
    Ok((i, AddrOffset::Imm(imm as i32)))
}

/// Parse a non post-indexed offset, i.e. one appearing between the square brackets, and addressing
/// mode
/// e.g. [r0, r1, lsl #123]!
///         ^ -------------^ parses this span
fn pre_offset(i: &str) -> IResult<&str, (AddrOffset, AddrMode)> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, offset) = alt((imm_offset, index_offset))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = match_char(']')(i)?;
    let (i, excl) = opt(match_char('!'))(i)?;
    let mode = match excl {
        Some(_) => AddrMode::PreIndex,
        None => AddrMode::Offset,
    };
    Ok((i, (offset, mode)))
}

/// Parse a post-index offset, i.e. one appearing after the square brackets
/// e.g. [r0], r1, ROR #32
///         ^------------^ parses this span
fn post_offset(i: &str) -> IResult<&str, (AddrOffset, AddrMode)> {
    let (i, _) = match_char(']')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, offset) = alt((imm_offset, index_offset))(i)?;
    Ok((i, (offset, AddrMode::PostIndex)))
}

fn address(input: &str) -> IResult<&str, (Address, Option<AddrOffset>)> {
    let (i, _) = tag("[")(input)?;
    let (i, _) = multispace0(i)?;
    let (i, base) = register(i)?;
    let (i, _) = multispace0(i)?;
    let (i, result) = opt(alt((pre_offset, post_offset)))(i)?;
    let (i, offset, mode) = match result {
        None => {
            let (i, _) = multispace0(i)?;
            let (i, _) = match_char(']')(i)?;
            (i, None, AddrMode::Offset)
        }
        Some((o, m)) => (i, Some(o), m),
    };
    Ok((i, (Address { base, mode }, offset)))
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::*;

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(mnemonic("MLSOTHERTEXT"), Ok(("OTHERTEXT", (Op::MLS, Cond::AL, false))));
        assert_eq!(mnemonic("MLSLSREST"), Ok(("REST", (Op::MLS, Cond::LS, false))));
        assert_eq!(mnemonic("MLSLSS ..."), Ok((" ...", (Op::MLS, Cond::LS, true))));
        let res = mnemonic("QSDFIE ...");
        // let e = json::<VerboseError<&str>>(res).finish().err().unwrap();
        println!(
            "verbose errors - `json::<VerboseError<&str>>(res)`:\n{}",
            convert_error("QSDFIE ...", res.unwrap_err())
        );
    }

    #[test]
    fn test_parse_address() {
        let (rest, (addr, offset)) = address("[r0, r1, LSL #19]!..REST").unwrap();
        let (rest, (addr, offset)) = address("[PC, LR, ROR #20]..REST").unwrap();
        let (rest, (addr, offset)) = address("[r12, r9, ASR #1]..REST").unwrap();
        let (rest, (addr, offset)) = address("[r12, r9]..REST").unwrap();
        let (rest, (addr, offset)) = address("[r3, sp]!..REST").unwrap();
        let (rest, (addr, offset)) = address("[r12, #1932]..REST").unwrap();
        let (rest, (addr, offset)) = address("[r0, #-123]!..REST").unwrap();
        let (rest, (addr, offset)) = address("[r0]..REST").unwrap();
        let (rest, (addr, offset)) = address("[r0], r0..REST").unwrap();
        let (rest, (addr, offset)) = address("[r0], r0, LSR #23..REST").unwrap();
        assert!(address("[r1, r2, #123]").is_err());
        assert!(address("[r1]!, r2").is_err());
        assert!(address("[r1, r0, r3]!").is_err());
        assert!(address("[r1, r0, ROR r9]!").is_err());
    }
}
