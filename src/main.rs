use ndsjit::asm::alloc::RegAllocation;
// use ndsjit::disasm::try_disasm_arm;
use ndsjit::ir::Instr;
// use ndsjit::asm::execbuffer::ExecBuffer;
use ndsjit::asm::AssemblerX64;
// use std::fs;
use std::io::Error;
use std::vec::Vec;



// fn assemble_instr_buffer(buff: Vec<Instr>) -> ExecBuffer {
    // let alloc = RegAllocation::default();
    // let asm = AssemblerX64::new(alloc);
    // for instr in buff {
	// asm.assemble(instr);
    // }
    // asm.get_exec_buffer()
// }

// When retrieving code to run for a given address there are 3 possibilities
// Compiled, uncompiled, and uncompileable/requires interpretation 


fn main() -> Result<(), Error> {
    // Load bin
    // initialize the CPU/registers
    // Main loop:
    //   Check code cache at location of PC
    //   if cached code exists, run code (transfer state, enter & run, transfer state back)
    //   else
    //     instruction at PC, disassemble and add to list of instructions
    //     once a runtime determined branch is reached stop and compile, add to cache, run
    // 
    // How to indicate that runtime info is needed? Basically need to use interpreted mode.
    // Maybe just set some kind of flag indicating that interpreted mode is required for next
    // instruction
    // OR - can we just jump straight to the interpreter code from the compiled code?
    //
    //
    
    // let f = fs::read("gba_bios/gba_bios.bin")?;
    // for i in 0..30 {
        // for j in 0..4 {
            // print!("{:02x?} ", &f[4 * i + j]);
        // }
        // let res = try_disasm_arm(0, &f[4 * i + j..4 * i + j + 4]);
        // println!();
    // }
    Ok(())
}
