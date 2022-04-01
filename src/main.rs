#![allow(dead_code, unused_variables)]

mod asm;
mod cpu;
mod ir;

use asm::alloc::RegAllocation;
use asm::AssemblerX64;
use cpu::VirtualState;
use ir::Instr;
use ir::Opcode::MOV;
use ir::Operand::Reg;
use ir::VReg::*;

fn main() {
    println!("Hello world");
    let mut asm = AssemblerX64::new(RegAllocation::default());
    let mov = Instr {
        opcode: MOV,
        operands: [Some(Reg(R0)), Some(Reg(R8)), None],
    };
    asm.gen_prologue().emit(mov).gen_epilogue();
    let func = asm.get_exec_buffer();

    let mut cpu = VirtualState::new();
    cpu.vregs[8] = 84859393;
    dbg!(cpu.vregs);
    func.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
    dbg!(cpu.vregs);
}
