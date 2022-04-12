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
    match bits(instr, 9..15) {
        // 0b01001XX
        0b0100100 | 0b0100101 | 0b0100110 | 0b0100111 => ldr_literal_thumb(addr, instr),
        // 0b01100XX
        0b0110000 | 0b0110001 | 0b0110010 | 0b0110011 => str_imm_thumb(instr),
        // 0b01101XX
        0b0110100 | 0b0110101 | 0b0110110 | 0b0110111 => ldr_imm_thumb(instr),
        // 0b01110XX
        0b0111000 | 0b0111001 | 0b0111010 | 0b0111011 => strb_imm_thumb(instr),
        // 0b01111XX
        0b0111100 | 0b0111101 | 0b0111110 | 0b0111111 => ldrb_imm_thumb(instr),
        // 0b10010XX
        0b1001000 | 0b1001001 | 0b1001010 | 0b1001011 => str_imm_sp_thumb(instr),
        // 0b10011XX
        0b1001100 | 0b1001101 | 0b1001110 | 0b1001111 => ldr_imm_sp_thumb(instr),
        0b0101000 => str_reg_thumb(instr),
        0b0101010 => strb_reg_thumb(instr),
        0b0101100 => ldr_reg_thumb(instr),
        0b0101110 => ldrb_reg_thumb(instr),
        // 0b10000XX
        0b1000000 | 0b1000001 | 0b1000010 | 0b1000011 => strh_imm_thumb(instr),
        // 0b10001XX
        0b1000100 | 0b1000101 | 0b1000110 | 0b1000111 => ldrh_imm_thumb(instr),
        0b0101001 => strh_reg_thumb(instr),
        0b0101011 => ldrsb_reg_thumb(instr),
        0b0101101 => ldrh_reg_thumb(instr),
        0b0101111 => ldrsh_reg_thumb(instr),
        _ => panic!("Unknown instruction"),
    }
}

#[cfg(test)]
mod tests;
