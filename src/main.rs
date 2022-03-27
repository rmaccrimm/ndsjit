#![allow(dead_code, unused_variables)]

mod asm;
mod cpu;
mod ir;

use asm::{AssemblerX64, VirtualState};

fn main() {
    let mut asm = AssemblerX64::new();

    asm.gen_prologue().gen_epilogue();

    let buf = asm.get_exec_buffer();
    let func = buf.as_func_ptr();

    let mut state = VirtualState::default();
    println!("{}", state.mem.len());
    println!("before: {}", state.vregs[0]);
    unsafe {
        func(state.vregs.as_mut_ptr());
    }
    println!("after: {}", state.vregs[0]);
}
