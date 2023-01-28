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
    error::{context, convert_error, Error, ErrorKind, ParseError, VerboseError},
    Err, IResult, Needed,
};
// use strum::ParseError;

/// Attempt to parse an opcode, starting with the longest possible (8 chars) and moving to the
/// shortest (1 char)
fn op(input: &str) -> IResult<&str, Op, VerboseError<&str>> {
    context(
        "op",
        alt((
            map_res(take(8usize), Op::from_str),
            map_res(take(7usize), Op::from_str),
            map_res(take(6usize), Op::from_str),
            map_res(take(5usize), Op::from_str),
            map_res(take(4usize), Op::from_str),
            map_res(take(3usize), Op::from_str),
            map_res(take(2usize), Op::from_str),
            map_res(take(1usize), Op::from_str),
        )),
    )(input)
}

fn cond(input: &str) -> IResult<&str, Cond, VerboseError<&str>> {
    map_res(take(2usize), Cond::from_str)(input)
}

fn register(input: &str) -> IResult<&str, Register, VerboseError<&str>> {
    map_res(alphanumeric1, Register::from_str)(input)
}

fn imm_val(i: &str) -> IResult<&str, u32, VerboseError<&str>> {
    let (i, _) = match_char('#')(i)?;
    let (i, val) = match_u32(i)?;
    Ok((i, val))
}

fn mnemonic(i: &str) -> IResult<&str, (Op, Cond, bool), VerboseError<&str>> {
    let (i, op) = op(i)?;
    let (i, cond) = opt(cond)(i)?;
    let (i, s) = opt(one_of("sS"))(i)?;
    Ok((i, (op, cond.unwrap_or(Cond::AL), s.is_some())))
}

/// Parses an immediate shift, starting from the comma following an index register
/// e.g. [r0, r1, lsl #123]!
///             ^--------^ parses this span
fn imm_shift(i: &str) -> IResult<&str, ImmShift, VerboseError<&str>> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, op) = map_res(take(3usize), ShiftOp::from_str)(i)?;
    let (i, _) = multispace1(i)?;
    let (i, imm) = imm_val(i)?;
    Ok((i, ImmShift { op, imm }))
}

/// Parse an index register offset with optional shift
fn index_offset(i: &str) -> IResult<&str, AddrOffset, VerboseError<&str>> {
    let (i, reg) = register(i)?;
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, shift) = opt(imm_shift)(i)?;
    Ok((i, AddrIndex { reg, shift }.into()))
}

// Parse an immediate address offset value (signed 32-bit)
fn imm_offset(i: &str) -> IResult<&str, AddrOffset, VerboseError<&str>> {
    let (i, imm) = imm_val(i)?;
    Ok((i, AddrOffset::Imm(imm as i32)))
}

/// Parse a non post-indexed offset, i.e. one appearing between the square brackets, and addressing
/// mode
/// e.g. [r0, r1, lsl #123]!
///         ^ -------------^ parses this span
fn pre_offset(i: &str) -> IResult<&str, (AddrOffset, AddrMode), VerboseError<&str>> {
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
fn post_offset(i: &str) -> IResult<&str, (AddrOffset, AddrMode), VerboseError<&str>> {
    let (i, _) = match_char(']')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, offset) = alt((imm_offset, index_offset))(i)?;
    Ok((i, (offset, AddrMode::PostIndex)))
}

fn address(input: &str) -> IResult<&str, (Address, Option<AddrOffset>), VerboseError<&str>> {
    let (i, _) = match_char('[')(input)?;
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
    use super::*;
    use crate::disasm::armv4t::{AddrMode::*, Register::*, ShiftOp::*};

    #[test]
    fn test_parse_op() {
        assert_eq!(op("VQRDMULH.."), Ok(("..", Op::VQRDMULH)));
        assert_eq!(op("B.."), Ok(("..", Op::B)));
        assert_eq!(op("BX.."), Ok(("..", Op::BX)));
    }

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(mnemonic("MLSOTHERTEXT"), Ok(("OTHERTEXT", (Op::MLS, Cond::AL, false))));
        assert_eq!(mnemonic("MLSLSREST"), Ok(("REST", (Op::MLS, Cond::LS, false))));
        assert_eq!(mnemonic("MLSLSS ..."), Ok((" ...", (Op::MLS, Cond::LS, true))));
    }

    #[test]
    fn test_parse_address() {
        let (rest, (addr, offset)) = address("[r0, r1, LSL #19]!..REST").unwrap();
        assert_eq!(rest, "..REST");
        assert_eq!(addr, Address { base: R0, mode: PreIndex });
        assert_eq!(
            offset.unwrap(),
            AddrIndex { reg: R1, shift: Some(ImmShift { op: LSL, imm: 19 }) }.into()
        );

        let (_, (addr, offset)) = address("[PC, LR, ROR #20]..REST").unwrap();
        assert_eq!(addr, Address { base: PC, mode: Offset });
        assert_eq!(
            offset.unwrap(),
            AddrIndex { reg: LR, shift: Some(ImmShift { op: ROR, imm: 20 }) }.into()
        );

        let (_, (addr, offset)) = address("[r12, r9, ASR #1]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: Offset });
        assert_eq!(
            offset.unwrap(),
            AddrIndex { reg: R9, shift: Some(ImmShift { op: ASR, imm: 1 }) }.into()
        );
        let (_, (addr, offset)) = address("[r12, r9]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: Offset });
        assert_eq!(offset.unwrap(), AddrIndex { reg: R9, shift: None }.into());

        let (_, (addr, offset)) = address("[r3, sp]!..REST").unwrap();
        assert_eq!(addr, Address { base: R3, mode: PreIndex });
        assert_eq!(offset.unwrap(), AddrIndex { reg: R3, shift: None }.into());

        let (_, (addr, offset)) = address("[r12, #1932]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: Offset });
        assert_eq!(offset.unwrap(), AddrOffset::Imm(1932));

        let (_, (addr, offset)) = address("[r0, #-123]!..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: PreIndex });
        assert_eq!(offset.unwrap(), AddrOffset::Imm(-123));

        let (_, (addr, offset)) = address("[r0]..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: Offset });
        assert_eq!(offset, None);

        let (_, (addr, offset)) = address("[r0], r0..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: PostIndex });
        assert_eq!(offset.unwrap(), AddrIndex { reg: R0, shift: None }.into());

        let (_, (addr, offset)) = address("[r0], r0, LSR #23..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: PostIndex });
        assert_eq!(
            offset.unwrap(),
            AddrIndex { reg: R0, shift: Some(ImmShift { op: LSR, imm: 23 }) }.into()
        );

        assert!(address("[r1, r2, #123]").is_err());
        assert!(address("[r1]!, r2").is_err());
        assert!(address("[r1, r0, r3]!").is_err());
        assert!(address("[r1, r0, ROR r9]!").is_err());
    }
}
