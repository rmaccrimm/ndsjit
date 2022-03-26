#![allow(dead_code, unused_variables)]

mod execbuffer;

use execbuffer::ExecBuffer;

use std::mem;

#[derive(PartialEq, Eq, Hash)]
enum VReg {
    R0,
    R1,
    R2,
}

#[derive(PartialEq, Eq, Hash)]
enum PhysRegX64 {
    EAX,
    EBX,
    EDX,
}

fn mov_reg_bptr(v: &mut Vec<u8>) {}

fn main() {
    println!("Hello, world!");

    // let mut buffer = [0u8; 10];
    let mut vregs: [u64; 3] = [1, 0, 0];

    let mut code: Vec<u8> = Vec::new();
    // mov rax, [rcx]  -- calling convention should pass first param as rcx
    code.push(0x48);
    code.push(0x8b);
    code.push(0x01);
    // add %rax, 32003
    code.push(0x48);
    code.push(0x05);
    code.push(0x00);
    code.push(0x00);
    code.push(0x00);
    code.push(0x10);
    // // mov [%rcx], %rax
    code.push(0x48);
    code.push(0x89);
    code.push(0x01);
    // ret
    code.push(0xc3);

    let buf = ExecBuffer::from_vec(code).unwrap();
    println!("before: {}", vregs[0]);
    unsafe {
        let func: unsafe extern "C" fn(*mut u64) = mem::transmute(buf.ptr);
        func(vregs.as_mut_ptr());
    }
    println!("after: {}", vregs[0]);
}
