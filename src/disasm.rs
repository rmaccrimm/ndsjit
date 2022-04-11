mod armv4t;
use super::ir::Opcode;
use armv4t::*;
use std::ops::Range;

fn bits(word: u32, r: Range<usize>) -> u32 {
    assert!(r.start < r.end);
    (word >> r.start) & ((1 << (r.end - r.start + 1)) - 1)
}

fn bit(word: u32, b: usize) -> bool {
    (word >> b) & 1 == 1
}

pub fn try_disasm_arm(addr: u32, instr_bytes: &[u8; 4]) -> Option<Opcode> {
    let instr = u32::from_le_bytes(*instr_bytes);
    None
}

pub fn try_disasm_thumb(addr: u32, instr_bytes: &[u8; 2]) -> Option<Opcode> {
    let instr = u16::from_le_bytes(*instr_bytes) as u32;
    match bits(instr, 13..15) {
        0b010 => match bits(instr, 11..12) {
            0b00 => None, // HiReg/BX
            _ => Some(disasm_ldr_str_thumb(addr, instr)),
        },
        0b011 | 0b100 => Some(disasm_ldr_str_thumb(addr, instr)),
        _ => None,
    }
}

fn disasm_ldr_str_arm(addr: u32, instr: u32) -> Opcode {
    panic!()
}

fn disasm_ldr_str_thumb(addr: u32, instr: u32) -> Opcode {
    if bits(instr, 11..15) == 0b01001 {
        return ldr_literal_thumb(addr, instr);
    } else if bits(instr, 13..15) == 0b011 {
        return match bits(instr, 11..12) {
            0b00 => str_imm_thumb(instr),
            0b01 => ldr_imm_thumb(instr),
            0b10 => strb_imm_thumb(instr),
            0b11 => ldrb_imm_thumb(instr),
            _ => panic!(),
        };
    } else if bits(instr, 12..15) == 0b1001 {
        return match bit(instr, 11) {
            true => ldr_imm_sp_thumb(instr),
            false => str_imm_sp_thumb(instr),
        };
    } else if bits(instr, 12..15) == 0b0101 && !bit(instr, 9) {
        return match bits(instr, 10..11) {
            0b00 => str_reg_thumb(instr),
            0b01 => strb_reg_thumb(instr),
            0b10 => ldr_reg_thumb(instr),
            0b11 => ldrb_reg_thumb(instr),
            _ => panic!(),
        };
    } else if bits(instr, 12..15) == 0b1000 {
        return match bit(instr, 11) {
            true => ldrh_imm_thumb(instr),
            false => strh_imm_thumb(instr),
        };
    } else if bits(instr, 12..15) == 0b0101 && bit(instr, 9) {
        return match bits(instr, 10..11) {
            0b00 => strh_reg_thumb(instr),
            0b01 => ldrsb_reg_thumb(instr),
            0b10 => ldrh_reg_thumb(instr),
            0b11 => ldrsh_reg_thumb(instr),
            _ => panic!(),
        };
    }
    panic!();
}

#[cfg(test)]
mod tests;
