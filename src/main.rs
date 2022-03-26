#![allow(dead_code, unused_variables)]

mod asm;
mod execbuffer;

use asm::AssemblerX64;
use execbuffer::ExecBuffer;

pub struct VirtualState {
    pub vregs: [u64; 30],
    pub mem: Box<[u8]>,
}

impl Default for VirtualState {
    fn default() -> VirtualState {
        let mem_size = 4 * (1 << 20);
        VirtualState {
            vregs: [0; 30],
            mem: vec![0; mem_size].into_boxed_slice(),
        }
    }
}

fn main() {
    let mut asm = AssemblerX64::default();
    asm.gen_prologue().add_rax_imm32(1932).gen_epilogue();

    let buf = ExecBuffer::from_vec(asm.code).unwrap();
    let func = buf.as_func_ptr();

    let mut state = VirtualState::default();
    println!("{}", state.mem.len());
    println!("before: {}", state.vregs[0]);
    unsafe {
        func(state.vregs.as_mut_ptr());
    }
    println!("after: {}", state.vregs[0]);
}
