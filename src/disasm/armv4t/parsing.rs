use std::str::FromStr;

use crate::disasm::armv4t::AddrIndex;

use super::{
    AddrMode, AddrOffset, Address, Cond, ExtraOperand, ImmShift, Instruction, Op, Operand,
    RegShift, Register, Shift, ShiftOp,
};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take},
    character::complete::{
        alphanumeric1, char as match_char, i32 as match_i32, multispace0, multispace1, one_of,
        u32 as match_u32,
    },
    combinator::{map, map_res, opt},
    error::{context, VerboseError},
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

pub type ParseResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

/// Attempt to parse an opcode, starting with the longest possible (8 chars) and moving to the
/// shortest (1 char)
fn op(input: &str) -> ParseResult<Op> {
    context(
        "Op",
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

fn mnemonic(i: &str) -> ParseResult<(Op, Cond, bool)> {
    let (i, op) = op(i)?;
    let (i, cond) = opt(cond)(i)?;
    let (i, s) = opt(one_of("sS"))(i)?;
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
fn reg_shift(i: &str) -> ParseResult<RegShift> {
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, op) = shift_op(i)?;
    let (i, _) = multispace1(i)?;
    let (i, reg) = register(i)?;
    Ok((i, RegShift { op, reg }))
}

/// Parse an index register offset with optional shift
fn index_offset(i: &str) -> ParseResult<AddrOffset> {
    let (i, reg) = register(i)?;
    let (i, shift) = opt(imm_shift)(i)?;
    Ok((i, AddrIndex { reg, shift }.into()))
}

// Parse an immediate address offset value (signed 32-bit)
fn imm_offset(i: &str) -> ParseResult<AddrOffset> {
    let (i, _) = match_char('#')(i)?;
    let (i, neg) = opt(match_char('-'))(i)?;
    let (i, imm) = match_i32(i)?;
    let sign = if neg.is_some() { -1 } else { 1 };
    Ok((i, AddrOffset::Imm(imm * sign)))
}

/// Parse a non post-indexed offset, i.e. one appearing between the square brackets, and addressing
/// mode
/// e.g. [r0, r1, lsl #123]!
///         ^ -------------^ parses this span
fn pre_offset(i: &str) -> ParseResult<(AddrOffset, AddrMode)> {
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
fn post_offset(i: &str) -> ParseResult<(AddrOffset, AddrMode)> {
    let (i, _) = match_char(']')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = match_char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, offset) = alt((imm_offset, index_offset))(i)?;
    Ok((i, (offset, AddrMode::PostIndex)))
}

fn address(input: &str) -> ParseResult<(Address, Option<AddrOffset>)> {
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

fn shifted_reg(i: &str) -> ParseResult<(Register, Option<Shift>)> {
    let shift =
        alt((map(reg_shift, Shift::Reg), map(imm_shift, Shift::Imm), map(rrx_shift, Shift::Imm)));
    let (i, reg) = register(i)?;
    let (i, shift) = opt(shift)(i)?;
    Ok((i, (reg, shift)))
}

fn operand(i: &str) -> ParseResult<(Operand, Option<ExtraOperand>)> {
    let reg = map(shifted_reg, |(r, s)| (Operand::Reg(r), s.map(ExtraOperand::Shift)));
    let addr = map(address, |(a, o)| (Operand::Addr(a), o.map(ExtraOperand::Offset)));
    let imm = map(imm_val, |i| (Operand::Imm(i), None));
    context("Operand", alt((reg, addr, imm)))(i)
}

/// Parses a single ARM instruction (in UAL syntax) into structured format
pub fn instruction(i: &str) -> ParseResult<Instruction> {
    let (i, (op, cond, mut set_flags)) = mnemonic(i)?;

    if [Op::TEQ, Op::TST, Op::CMN, Op::CMP].contains(&op) {
        set_flags = true;
    }

    let (i, _) = multispace1(i)?;

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
    use crate::disasm::armv4t::{
        AddrMode::*, AddrOffset::*, Cond::*, ExtraOperand, Op::*, Operand::*, Register::*, ShiftOp,
    };

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
            AddrIndex { reg: R1, shift: Some(ImmShift { op: ShiftOp::LSL, imm: 19 }) }.into()
        );

        let (_, (addr, offset)) = address("[PC, LR, ROR #20]..REST").unwrap();
        assert_eq!(addr, Address { base: PC, mode: Offset });
        assert_eq!(
            offset.unwrap(),
            AddrIndex { reg: LR, shift: Some(ImmShift { op: ShiftOp::ROR, imm: 20 }) }.into()
        );

        let (_, (addr, offset)) = address("[r12, r9, ASR #1]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: Offset });
        assert_eq!(
            offset.unwrap(),
            AddrIndex { reg: R9, shift: Some(ImmShift { op: ShiftOp::ASR, imm: 1 }) }.into()
        );
        let (_, (addr, offset)) = address("[r12, r9]..REST").unwrap();
        assert_eq!(addr, Address { base: R12, mode: Offset });
        assert_eq!(offset.unwrap(), AddrIndex { reg: R9, shift: None }.into());

        let (_, (addr, offset)) = address("[r3, sp]!..REST").unwrap();
        assert_eq!(addr, Address { base: R3, mode: PreIndex });
        assert_eq!(offset.unwrap(), AddrIndex { reg: SP, shift: None }.into());

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
            AddrIndex { reg: R0, shift: Some(ImmShift { op: ShiftOp::LSR, imm: 23 }) }.into()
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
                operands: vec![Reg(R0), Addr(Address { base: R1, mode: PreIndex },),],
                extra: Some(ExtraOperand::Offset(Index(AddrIndex {
                    reg: R2,
                    shift: Some(ImmShift { op: ShiftOp::LSL, imm: 92 },),
                },),),),
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
