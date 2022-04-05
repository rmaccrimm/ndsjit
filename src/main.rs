#![allow(dead_code, unused_variables)]

mod asm;
mod cpu;
mod ir;

use std::mem;

use asm::alloc::RegAllocation;
use asm::AssemblerX64;
use cpu::VirtualState;
use ir::Operand;
use ir::Operand::{Imm, Ptr, Reg};
use ir::VReg::*;

fn main() {
    dbg!(mem::size_of::<Operand>());
    let mut asm = AssemblerX64::new(RegAllocation::default());
    asm.gen_prologue()
        .mov_imm(PC, 8323)
        .mov_reg(R0, PC)
        .mov_imm(R11, 100)
        .gen_epilogue();
    asm.hex_dump();
    let func = asm.get_exec_buffer();

    let mut cpu = VirtualState::new();
    dbg!(cpu.vregs);
    func.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
    dbg!(cpu.vregs);
}
