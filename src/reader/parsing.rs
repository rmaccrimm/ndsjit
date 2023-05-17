use std::str::FromStr;

use crate::ir::{
    AddrMode, Address, Cond, ExtraOperand, ImmShift, Instruction, Offset, Op, Operand, Register,
    Shift, ShiftOp,
};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take},
    character::complete::{
        alphanumeric1, char as match_char, multispace0, multispace1, one_of, u32 as match_u32,
    },
    combinator::{map, map_res, opt},
    error::{context, VerboseError},
    multi::separated_list1,
    sequence::{terminated, tuple},
    IResult,
};

pub type ParseResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

fn cond(input: &str) -> ParseResult<Cond> {
    context("Cond", map_res(take(2usize), Cond::from_str))(input)
}

fn register(input: &str) -> ParseResult<Register> {
    context("Register", map_res(alphanumeric1, Register::from_str))(input)
}

fn shift_op(input: &str) -> ParseResult<ShiftOp> {
    context("ShiftOp", map_res(take(3usize), ShiftOp::from_str))(input)
}

fn imm_val(i: &str) -> ParseResult<u32> {
    let (i, (_, val)) = context("imm_val", tuple((match_char('#'), match_u32)))(i)?;
    Ok((i, val))
}

/// Parses the instruction mnemonic and the whitespace which must follow it, e.g. "ADDEQS "
fn mnemonic(i: &str) -> ParseResult<(Op, Cond, bool)> {
    let op = |x: usize| map_res(take(x), Op::from_str);
    let mnem = |x: usize| terminated(tuple((op(x), opt(cond), opt(one_of("sS")))), multispace1);
    // Have to go through every possible length of op until we succesfully parse everything, up to
    // the terminating whitespace
    let mut parse = alt((mnem(8), mnem(7), mnem(6), mnem(5), mnem(4), mnem(3), mnem(2), mnem(1)));
    let (i, (op, cond, s)) = parse(i)?;
    Ok((i, (op, cond.unwrap_or(Cond::AL), s.is_some())))
}

/// Parses an immediate shift, starting from the comma following a base register
/// e.g. [r0, r1, lsl #123]!
///             ^--------^ parses this span
fn imm_shift(i: &str) -> ParseResult<ImmShift> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, op) = shift_op(i)?;
    let (i, _) = multispace0(i)?;
    let (i, imm) = imm_val(i)?;
    Ok((i, ImmShift { op, imm }))
}

/// Isn't followed by another operand, unlike other shifts
fn rrx_shift(i: &str) -> ParseResult<ImmShift> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, op) = tag_no_case("RRX")(i)?;
    Ok((i, ImmShift { op: op.parse().unwrap(), imm: 1 }))
}

/// Parses a register shift, starting from the comma following a base register
/// e.g. ADR r0, r1, r2, lsl r3
///                    ^------^ parses this span
fn reg_shift(i: &str) -> ParseResult<Shift> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, op) = shift_op(i)?;
    let (i, _) = multispace1(i)?;
    let (i, reg) = register(i)?;
    Ok((i, Shift::reg(op, reg)))
}

/// Parse an index register offset with optional shift
fn reg_offset(i: &str) -> ParseResult<Offset> {
    let (i, neg) = opt(match_char('-'))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, reg) = register(i)?;
    let (i, shift) = opt(alt((imm_shift, rrx_shift)))(i)?;
    Ok((i, Offset::reg(reg, shift, neg.is_none())))
}

// Parse an immediate address offset value (signed 32-bit)
fn imm_offset(i: &str) -> ParseResult<Offset> {
    let (i, _) = match_char('#')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, neg) = opt(match_char('-'))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, imm) = match_u32(i)?;
    Ok((i, Offset::imm(imm, neg.is_none())))
}

/// Parse a non post-indexed offset, i.e. one appearing between the square brackets, and addressing
/// mode
/// e.g. [r0, r1, lsl #123]!
///         ^ -------------^ parses this span
fn pre_offset(i: &str) -> ParseResult<(ExtraOperand, AddrMode)> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, offset) =
        alt((map(imm_offset, ExtraOperand::from), map(reg_offset, ExtraOperand::from)))(i)?;
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
fn post_offset(i: &str) -> ParseResult<(ExtraOperand, AddrMode)> {
    let (i, _) = match_char(']')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, offset) =
        alt((map(imm_offset, ExtraOperand::from), map(reg_offset, ExtraOperand::from)))(i)?;
    Ok((i, (offset, AddrMode::PostIndex)))
}

fn address(input: &str) -> ParseResult<(Address, Option<ExtraOperand>)> {
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

fn shifted_reg(i: &str) -> ParseResult<(Register, Option<ExtraOperand>)> {
    let shift = alt((
        map(reg_shift, ExtraOperand::from),
        map(imm_shift, ExtraOperand::from),
        map(rrx_shift, ExtraOperand::from),
    ));
    let (i, reg) = register(i)?;
    let (i, shift) = opt(shift)(i)?;
    Ok((i, (reg, shift)))
}

fn operand(i: &str) -> ParseResult<(Operand, Option<ExtraOperand>)> {
    let reg = map(shifted_reg, |(r, s)| (Operand::Reg(r), s.map(ExtraOperand::from)));
    let addr = map(address, |(a, o)| (Operand::Addr(a), o.map(ExtraOperand::from)));
    let imm = map(imm_val, |i| (Operand::Imm(i), None));
    context("Operand", alt((reg, addr, imm)))(i)
}

/// Parses a single ARM instruction (in UAL syntax) into structured format
pub fn instruction(i: &str) -> ParseResult<Instruction> {
    let (i, (op, cond, mut set_flags)) = mnemonic(i)?;

    if [Op::TEQ, Op::TST, Op::CMN, Op::CMP].contains(&op) {
        set_flags = true;
    }

    let sep = tuple((multispace0, match_char(','), multispace0));

    // NOTE - issue with parsing: currently having more than 1 extra operand is considered a valid
    // parse. Not sure if that can be detected
    let (i, res) = separated_list1(sep, operand)(i)?;
    let operands = res.iter().map(|x| x.0).collect();
    let extra = res.iter().map(|x| x.1).find(Option::is_some).flatten();

    Ok((i, Instruction { op, cond, set_flags, operands, extra }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Cond::*, Op::*, Operand::*, Register::*};

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(mnemonic("MLS OTHERTEXT"), Ok(("OTHERTEXT", (Op::MLS, Cond::AL, false))));
        assert_eq!(mnemonic("MLSLS REST"), Ok(("REST", (Op::MLS, Cond::LS, false))));
        assert_eq!(mnemonic("MLSLSS ..."), Ok(("...", (Op::MLS, Cond::LS, true))));
        assert_eq!(mnemonic("LDRHI "), Ok(("", (Op::LDR, Cond::HI, false))));
    }

    #[test]
    fn test_parse_address() {
        let (rest, (addr, offset)) = address("[r0, r1, LSL #19]!..REST").unwrap();
        assert_eq!(rest, "..REST");
        assert_eq!(addr, Address { base: R0, mode: AddrMode::PreIndex });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R1, Some(ImmShift { op: ShiftOp::LSL, imm: 19 }), true,).into()
        );

        let (_, (addr, offset)) = address("[PC, LR, ROR #20]..REST").unwrap();
        assert_eq!(addr, Address { base: PC, mode: AddrMode::Offset });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(LR, Some(ImmShift { op: ShiftOp::ROR, imm: 20 }), true).into()
        );

        let (_, (addr, offset)) = address("[r12, r9, ASR #1]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: AddrMode::Offset });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R9, Some(ImmShift { op: ShiftOp::ASR, imm: 1 }), true).into()
        );
        let (_, (addr, offset)) = address("[r12, r9]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: AddrMode::Offset });
        assert_eq!(offset.unwrap(), Offset::reg(R9, None, true).into());

        let (_, (addr, offset)) = address("[r3, sp]!..REST").unwrap();
        assert_eq!(addr, Address { base: R3, mode: AddrMode::PreIndex });
        assert_eq!(offset.unwrap(), Offset::reg(SP, None, true).into());

        let (_, (addr, offset)) = address("[r12, #1932]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: AddrMode::Offset });
        assert_eq!(offset.unwrap(), Offset::imm(1932, true).into());

        let (_, (addr, offset)) = address("[r0, #-123]!..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::PreIndex });
        assert_eq!(offset.unwrap(), Offset::imm(123, false).into());

        let (_, (addr, offset)) = address("[r0]..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::Offset });
        assert_eq!(offset, None);

        let (_, (addr, offset)) = address("[r0], r0..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::PostIndex });
        assert_eq!(offset.unwrap(), Offset::reg(R0, None, true).into());

        let (_, (addr, offset)) = address("[r0], r0, LSR #23..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::PostIndex });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R0, Some(ImmShift { op: ShiftOp::LSR, imm: 23 }), true,).into()
        );
        let (_, (addr, offset)) = address("[r12, - r9, ASR #1]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: AddrMode::Offset });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R9, Some(ImmShift { op: ShiftOp::ASR, imm: 1 }), false,).into()
        );
        let (_, (addr, offset)) = address("[r0], -r0, LSR #23..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::PostIndex });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R0, Some(ImmShift { op: ShiftOp::LSR, imm: 23 }), false,).into()
        );
        let (_, (addr, offset)) = address("[r0, r1, rrx]..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::Offset });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R1, Some(ImmShift { op: ShiftOp::RRX, imm: 1 }), true,).into()
        );
        let (_, (addr, offset)) = address("[r0], -r1, rrx..REST").unwrap();
        assert_eq!(addr, Address { base: R0, mode: AddrMode::PostIndex });
        assert_eq!(
            offset.unwrap(),
            Offset::reg(R1, Some(ImmShift { op: ShiftOp::RRX, imm: 1 }), false,).into()
        );

        assert!(address("[r1, r2, #123]").is_err());
        assert!(address("[r1, r0, r3]").is_err());
        assert!(address("[r1, r0, ROR r9]").is_err());
    }

    #[test]
    fn test_parse_instr() {
        let (_, instr) = instruction("LDRLE r0, [r1, r2, LSL #92]!").unwrap();
        assert_eq!(
            instr,
            Instruction {
                cond: LE,
                op: LDR,
                operands: vec![
                    Reg(R0),
                    Addr(Address { base: R1, mode: AddrMode::PreIndex },),
                ],
                extra: Some(
                    Offset::reg(R2, Some(ImmShift { op: ShiftOp::LSL, imm: 92 }), true,).into()
                ),
                set_flags: false,
            }
        );
        let (_, instr) = instruction("UMAALHI r0, r1, lr, sp").unwrap();
        assert_eq!(
            instr,
            Instruction {
                cond: HI,
                op: UMAAL,
                operands: vec![Reg(R0), Reg(R1), Reg(LR), Reg(SP)],
                extra: None,
                set_flags: false,
            }
        );
        let (_, instr) = instruction("ADDS r1, r2, #9393").unwrap();
        assert_eq!(
            instr,
            Instruction {
                cond: AL,
                op: ADD,
                operands: vec![Reg(R1), Reg(R2), Operand::Imm(9393)],
                extra: None,
                set_flags: true,
            }
        );
        let (_, instr) = instruction("ADDS r1, r2, r3, RRX").unwrap();
        assert_eq!(
            instr,
            Instruction {
                cond: AL,
                op: ADD,
                operands: vec![Reg(R1), Reg(R2), Reg(R3)],
                extra: Some(ImmShift { imm: 1, op: ShiftOp::RRX }.into()),
                set_flags: true,
            }
        );
    }
}
