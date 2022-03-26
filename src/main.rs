#![allow(dead_code, unused_variables)]

mod execbuffer;

use execbuffer::ExecBuffer;

use std::collections::HashMap;

fn write_bytes(v: &mut Vec<u8>, bytes: &[u8]) {
    for &b in bytes {
        v.push(b);
    }
}

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

enum RegArm32 {
    R0,
    R1,
}

// Represents physical registers
enum RegX64 {
    RAX,
    RCX,
}

#[derive(Default)]
pub struct AssemblerX64 {
    pub code: Vec<u8>,
}

impl AssemblerX64 {
    // Initialize physical register values with those in virtual registers (looked up through pointer in %rcx) and set
    // up the stack. If we have to spill to memory, I guess that will make use of the (physical) stack?
    fn gen_prologue(&mut self, reg_allocation: HashMap<RegArm32, RegX64>) {
        self.mov_reg64_ptr64(RegX64::RAX, RegX64::RCX);
    }

    // Move physical register values back to virtual state (through pointer still stored in %rcx - maybe should move to
    // stack to free up another register?)
    fn gen_epilogue(&mut self, reg_allocation: HashMap<RegArm32, RegX64>) {
        self.mov_ptr64_reg64(RegX64::RCX, RegX64::RAX);
    }

    // These instructions will need to know which registers have spilled to memory, or should that be handled 1 layer up?
    fn mov_reg64_ptr64(&mut self, dest: RegX64, src: RegX64) {}

    fn mov_ptr64_reg64(&mut self, dest: RegX64, src: RegX64) {}
}

fn main() {
    let mut code: Vec<u8> = Vec::new();
    // mov rax, [rcx]  -- calling convention should pass first param as rcx
    write_bytes(&mut code, &[0x48, 0x8b, 0x01]);
    // add %rax, 32003
    write_bytes(&mut code, &[0x48, 0x05, 0x00, 0x00, 0x00, 0x10]);
    // mov [%rcx], %rax
    write_bytes(&mut code, &[0x48, 0x89, 0x01]);
    // ret
    code.push(0xc3);

    let buf = ExecBuffer::from_vec(code).unwrap();
    let func = buf.as_func_ptr();

    let mut state = VirtualState::default();
    println!("{}", state.mem.len());
    println!("before: {}", state.vregs[0]);
    unsafe {
        func(state.vregs.as_mut_ptr());
    }
    println!("after: {}", state.vregs[0]);
}
