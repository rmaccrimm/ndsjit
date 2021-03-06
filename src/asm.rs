pub mod alloc;
mod execbuffer;
pub mod vreg;
mod x64;
use std::mem;

use super::ir;
use alloc::RegAllocation;
use execbuffer::ExecBuffer;

pub struct AssemblerX64 {
    reg_alloc: RegAllocation,
}

const REG_SIZE: usize = mem::size_of::<u32>() as usize;

// Stack contains prev stack pointer, followed by spilled registers
const SPILL_START: i32 = -1 * mem::size_of::<u64>() as i32;

fn spill_stack_disp(ind: usize) -> i32 {
    SPILL_START - ((REG_SIZE * ind) as i32)
}

/*
Planned use - these methods won't be called directly to setup machine code, but instead the
emit/translate/assemble (tbd) function will be passed IR instructions to encode, and then the
get_exec_buffer will be called, something like:

    let func = AssemblerX64::new(reg_alloc)
        .emit(&instructions)
        .get_exec_buffer();
*/
impl AssemblerX64 {
    pub fn new(reg_alloc: RegAllocation) -> AssemblerX64 {
        let mut asm = AssemblerX64 { reg_alloc };
        asm.reg_alloc.gen_prologue();
        asm
    }

    pub fn get_exec_buffer(self) -> ExecBuffer {
        ExecBuffer::from_vec(self.reg_alloc.code.buf).unwrap()
    }

    pub fn hex_dump(&mut self) {
        self.reg_alloc.gen_epilogue();
        for b in self.reg_alloc.code.buf.iter() {
            print!("{:02x}", b);
        }
        println!();
    }

    fn mov_reg(&mut self, dest: ir::VReg, src: ir::VReg) -> &mut Self {
        self.reg_alloc.mov_reg(dest, src);
        self
    }

    fn mov_imm(&mut self, dest: ir::VReg, imm: i16) -> &mut Self {
        self.reg_alloc.mov_imm16(dest, imm);
        self
    }

    /// Load value to register from absolute address
    fn ldr_abs(&mut self, dest: ir::VReg, addr: u32) -> &mut Self {
        self.reg_alloc.mov_abs(dest, addr);
        self
    }

    /// Load value to register from address in pointer register plus immediate offset
    fn ldr_rel_imm(&mut self, dest: ir::VReg, base: ir::VReg, offset: i32) -> &mut Self {
        self.reg_alloc.mov_offset(dest, base, offset);
        self
    }

    /// Load value to register from address in pointer register plus index register
    fn ldr_rel_ind_imm(
        &mut self,
        dest: ir::VReg,
        base: ir::VReg,
        index: ir::VReg,
        offset: i32,
    ) -> &mut Self {
        self.reg_alloc.mov_index(dest, base, index, offset);
        self
    }

    fn ret(&mut self) -> &mut Self {
        self.reg_alloc.ret();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{AssemblerX64, RegAllocation};
    use crate::cpu::ARM7;
    use crate::ir::VReg::*;

    #[test]
    fn test_mov() {
        let mut cpu = ARM7::new();
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.mov_imm(R0, 4958) // phys
            .mov_reg(R6, R0) // phys -> phys
            .mov_imm(SP, 193) // spill
            .mov_reg(LR, SP) // spill -> spill
            .mov_reg(PC, R6) // phys -> spill
            .mov_reg(R3, PC); // spill -> phys
        asm.hex_dump();
        let f = asm.get_exec_buffer();
        dbg!(cpu.vregs);
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        dbg!(cpu.vregs);
        assert_eq!(cpu.vregs[R0 as usize], 4958);
        assert_eq!(cpu.vregs[R6 as usize], 4958);
        assert_eq!(cpu.vregs[SP as usize], 193);
        assert_eq!(cpu.vregs[LR as usize], 193);
        assert_eq!(cpu.vregs[PC as usize], 4958);
        assert_eq!(cpu.vregs[R3 as usize], 4958);
    }

    fn setup_cpu_test_data(cpu: &mut ARM7) {
        cpu.vregs[SP as usize] = 80;
        cpu.vregs[R3 as usize] = 81;
        cpu.vregs[R4 as usize] = 16;
        cpu.vregs[R11 as usize] = 4;
        cpu.mem[80] = 0xa3;
        cpu.mem[81] = 0x03;
        cpu.mem[82] = 0xf1;
        cpu.mem[83] = 0x4e;
        cpu.mem[84] = 0xbb;
        cpu.mem[85] = 0x73;
        cpu.mem[86] = 0xda;
        cpu.mem[87] = 0x09;
        cpu.mem[96] = 0x6c;
        cpu.mem[97] = 0x78;
        cpu.mem[98] = 0xff;
        cpu.mem[99] = 0x32;
        cpu.mem[10100] = 0x1a;
        cpu.mem[10101] = 0x6b;
        cpu.mem[10102] = 0x80;
        cpu.mem[10103] = 0xcc;
    }

    #[test]
    fn test_ldr_abs() {
        let mut cpu = ARM7::new();
        setup_cpu_test_data(&mut cpu);
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.ldr_abs(R0, 80) // phys
            .ldr_abs(PC, 98); // spill
        asm.hex_dump();
        dbg!(cpu.vregs);
        let f = asm.get_exec_buffer();
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        dbg!(cpu.vregs);
        assert_eq!(cpu.vregs[R0 as usize], 0x4ef103a3);
        assert_eq!(cpu.vregs[PC as usize], 0x000032ff);
    }

    #[test]
    fn test_ldr_rel_ind_imm() {
        let mut cpu = ARM7::new();
        setup_cpu_test_data(&mut cpu);
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.ldr_rel_ind_imm(R1, R3, R4, 0) // ppp
            .ldr_rel_ind_imm(R2, R3, R11, 0) // pps
            .ldr_rel_ind_imm(R5, SP, R4, 0) // psp
            .ldr_rel_ind_imm(R6, SP, R11, 0) // pss
            .ldr_rel_ind_imm(LR, SP, R4, 0) // ssp
            .ldr_rel_ind_imm(R7, SP, R11, 10016)
            .ldr_rel_ind_imm(R8, SP, R4, -10);
        asm.hex_dump();
        dbg!(cpu.vregs);
        let f = asm.get_exec_buffer();
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        assert_eq!(cpu.vregs[R1 as usize], 0x0032ff78);
        assert_eq!(cpu.vregs[R2 as usize], 0x0009da73);
        assert_eq!(cpu.vregs[R5 as usize], 0x32ff786c);
        assert_eq!(cpu.vregs[R6 as usize], 0x09da73bb);
        assert_eq!(cpu.vregs[LR as usize], 0x32ff786c);
        assert_eq!(cpu.vregs[R7 as usize], 0xcc806b1a);
        assert_eq!(cpu.vregs[R8 as usize], 0x000009da);
    }

    #[test]
    fn test_ldr_rel_imm() {
        let mut cpu = ARM7::new();
        setup_cpu_test_data(&mut cpu);
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.ldr_rel_imm(R9, R3, -1) // pp
            .ldr_rel_imm(R10, SP, 16) // ps
            .ldr_rel_imm(PC, R4, 64) // sp
            .ldr_rel_imm(R12, SP, 2); // ss
        asm.hex_dump();
        dbg!(cpu.vregs);
        let f = asm.get_exec_buffer();
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        assert_eq!(cpu.vregs[R9 as usize], 0x4ef103a3);
        assert_eq!(cpu.vregs[R10 as usize], 0x32ff786c);
        assert_eq!(cpu.vregs[PC as usize], 0x4ef103a3);
        assert_eq!(cpu.vregs[R12 as usize], 0x73bb4ef1);
    }
}
