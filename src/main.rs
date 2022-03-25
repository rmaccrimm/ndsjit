#![allow(dead_code, unused_variables)]

use std::mem;
use std::ptr;
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READ, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

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

    println!("{}", vregs[0]);
    unsafe {
        let mut system_info = SYSTEM_INFO::default();
        GetSystemInfo(&mut system_info as *mut SYSTEM_INFO);

        let buf = VirtualAlloc(
            ptr::null(),
            system_info.dwPageSize as usize,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        );
        ptr::copy_nonoverlapping(code.as_ptr(), buf as *mut u8, code.len());

        let mut dummy = PAGE_PROTECTION_FLAGS::default();
        VirtualProtect(
            buf,
            code.len(),
            PAGE_EXECUTE_READ,
            &mut dummy as *mut PAGE_PROTECTION_FLAGS,
        );

        let func: unsafe extern "C" fn(*mut u64) = mem::transmute(buf);
        func(vregs.as_mut_ptr());

        VirtualFree(buf, code.len(), MEM_RELEASE);
    }
    println!("{}", vregs[0]);
    // unsafe {
    // func();
    // func(vregs.as_mut_ptr());
    // }
}
