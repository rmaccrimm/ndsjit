use ndsjit::{
    asm::{alloc::RegAllocation, AssemblerX64},
    cpu::ARM7,
};

fn main() {
    let mut asm = AssemblerX64::new(RegAllocation::default());
    asm.call_rcx().ret();
    asm.hex_dump();
    let func = asm.get_exec_buffer();

    let mut cpu = ARM7::new();
    dbg!(cpu.vregs);
    func.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
    dbg!(cpu.vregs);
}
