use super::RegX64::*;
use super::*;

#[test]
fn test_mov_reg64_ptr64() {
    let mut code = EmitterX64::new();
    code.mov_reg64_ptr64(R8, RBP)
        .mov_reg64_ptr64(R15, RSI)
        .mov_reg64_ptr64(RDI, RBX)
        .mov_reg64_ptr64(RAX, RAX)
        .mov_reg64_ptr64(R11, RCX)
        .mov_reg64_ptr64(RBP, RSP)
        .mov_reg64_ptr64(RCX, RDI)
        .mov_reg64_ptr64(R9, R12)
        .mov_reg64_ptr64(RAX, R13);
    assert_eq!(
        code.buf,
        vec![
            0x4C, 0x8B, 0x45, 0x00, // mov r8 [rbp]
            0x4C, 0x8B, 0x3E, // mov r15, [rsi]
            0x48, 0x8B, 0x3B, // mov rdi,[rbx]
            0x48, 0x8B, 0x00, // mov rax,[rax]
            0x4C, 0x8B, 0x19, // mov r11,[rcx]
            0x48, 0x8B, 0x2C, 0x24, //mov rbp,[rsp]
            0x48, 0x8B, 0x0F, // mov rcx,[rdi]
            0x4D, 0x8B, 0x0C, 0x24, // mov r9,[r12]
            0x49, 0x8B, 0x45, 0x00 // mov rax,[r13]
        ]
    );
}

#[test]
fn test_mov_ptr64_reg64() {
    let mut code = EmitterX64::new();
    code.mov_ptr64_reg64(RBP, RDI)
        .mov_ptr64_reg64(RSP, RAX)
        .mov_ptr64_reg64(R12, R15)
        .mov_ptr64_reg64(R13, R13);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0x89, 0x7D, 0x00, // mov [rbp],rdi
            0x48, 0x89, 0x04, 0x24, // mov [rsp],rax
            0x4D, 0x89, 0x3C, 0x24, // mov [r12],r15
            0x4D, 0x89, 0x6D, 0x00, // mov r13,[r13]
        ]
    )
}

#[test]
fn test_mov_reg64_reg64() {
    let mut code = EmitterX64::new();
    code.mov_reg64_reg64(RBX, RDX)
        .mov_reg64_reg64(RDX, RBP)
        .mov_reg64_reg64(R9, RSP)
        .mov_reg64_reg64(RCX, R12);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0x89, 0xD3, // mov rbx,rdx
            0x48, 0x89, 0xEA, // mov rdx,rbp
            0x49, 0x89, 0xE1, // mov r9,rsp
            0x4C, 0x89, 0xE1, // mov rcx,r12
        ]
    )
}

#[test]
fn test_mov_reg64_ptr64_sib() {
    let mut code = EmitterX64::new();
    code.mov_reg64_ptr64_sib(RAX, RSP, R12, 8)
        .mov_reg64_ptr64_sib(RSP, RBP, RBP, 1)
        .mov_reg64_ptr64_sib(RDI, R12, R13, 4)
        .mov_reg64_ptr64_sib(RBX, RDX, RBP, 2)
        .mov_reg64_ptr64_sib(R13, R13, RAX, 1);
    assert_eq!(
        code.buf,
        vec![
            0x4A, 0x8B, 0x04, 0xE4, // mov rax, [rsp + r12*8]
            0x48, 0x8B, 0x64, 0x2D, 0x00, // mov rsp, [rbp + rbp]
            0x4B, 0x8B, 0x3C, 0xAC, // mov rdi, [r12 + 4*r13]
            0x48, 0x8B, 0x1C, 0x6A, // mov rbx, [rdx + 2*rbp]
            0x4D, 0x8B, 0x6C, 0x05, 0x00 // mov r13, [r13 + rax]
        ]
    );
}

#[test]
#[should_panic]
fn test_mov_reg64_ptr64_sib_rsp_index() {
    let mut code = EmitterX64::new();
    code.mov_reg64_ptr64_sib(RAX, RAX, RSP, 1);
}

#[test]
fn test_mov_reg64_ptr64_disp8() {
    let mut code = EmitterX64::new();
    code.mov_reg64_ptr64_disp8(R8, RBP, 127)
        .mov_reg64_ptr64_disp8(R9, RSP, -127)
        .mov_reg64_ptr64_disp8(R10, R12, 99)
        .mov_reg64_ptr64_disp8(R11, R13, -45)
        .mov_reg64_ptr64_disp8(RCX, R15, 109)
        .mov_reg64_ptr64_disp8(RBX, RAX, 12);
    assert_eq!(
        code.buf,
        vec![
            0x4C, 0x8B, 0x45, 0x7F, // mov r8,[rbp+127]
            0x4C, 0x8B, 0x4C, 0x24, 0x81, // mov r9, [rsp+10]
            0x4D, 0x8B, 0x54, 0x24, 0x63, // mov r10,[r12+99]
            0x4D, 0x8B, 0x5D, 0xd3, // mov r11,[r13+45]
            0x49, 0x8B, 0x4F, 0x6D, // mov rcx,[r15+109]
            0x48, 0x8B, 0x58, 0x0C, // mov rbx,[rax+12]
        ]
    )
}

#[test]
fn test_mov_ptr64_reg64_disp8() {
    let mut code = EmitterX64::new();
    code.mov_ptr64_reg64_disp8(RBP, RAX, -78)
        .mov_ptr64_reg64_disp8(RSP, RBX, 10)
        .mov_ptr64_reg64_disp8(R12, RCX, -3)
        .mov_ptr64_reg64_disp8(R13, R15, 44)
        .mov_ptr64_reg64_disp8(RDI, RSI, -1);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0x89, 0x45, 0xB2, // mov [rbp-78], rax
            0x48, 0x89, 0x5C, 0x24, 0x0A, // mov [rsp+10], rbx
            0x49, 0x89, 0x4C, 0x24, 0xFD, // mov [r12-3], rcx
            0x4D, 0x89, 0x7D, 0x2C, // mov [r13+44],r15
            0x48, 0x89, 0x77, 0xFF, // mov [rdi-1], rsi
        ]
    )
}

#[test]
fn test_mov_reg64_imm32() {
    let mut code = EmitterX64::new();
    code.mov_reg64_imm32(RAX, 485884)
        .mov_reg64_imm32(RBP, 0)
        .mov_reg64_imm32(RSP, 19)
        .mov_reg64_imm32(R12, 753432)
        .mov_reg64_imm32(R13, 458)
        .mov_reg64_imm32(R15, 2147483647)
        .mov_reg64_imm32(RSI, -28654);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0xC7, 0xC0, 0xFC, 0x69, 0x07, 0x00, // mov rax, 485884
            0x48, 0xC7, 0xC5, 0x00, 0x00, 0x00, 0x00, // mov rbp, 0
            0x48, 0xC7, 0xC4, 0x13, 0x00, 0x00, 0x00, // mov rsp, 19
            0x49, 0xC7, 0xC4, 0x18, 0x7F, 0x0B, 0x00, // mov r12, 753432
            0x49, 0xC7, 0xC5, 0xCA, 0x01, 0x00, 0x00, // mov r13, 458
            0x49, 0xC7, 0xC7, 0xFF, 0xFF, 0xFF, 0x7F, // mov r15, 2147483647
            0x48, 0xC7, 0xC6, 0x12, 0x90, 0xFF, 0xFF, // mov rsi, -28654
        ]
    )
}

#[test]
fn test_mov_ptr64_imm32() {
    let mut code = EmitterX64::new();
    code.mov_ptr64_imm32(RCX, -98)
        .mov_ptr64_imm32(RBP, 127)
        .mov_ptr64_imm32(RSP, -128)
        .mov_ptr64_imm32(R12, -0)
        .mov_ptr64_imm32(R13, 99)
        .mov_ptr64_imm32(R11, 2);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0xC7, 0x01, 0x9E, 0xFF, 0xFF, 0xFF, // movq [rcx], -98
            0x48, 0xC7, 0x45, 0x00, 0x7F, 0x00, 0x00, 0x00, // movq [rbp], 127
            0x48, 0xC7, 0x04, 0x24, 0x80, 0xFF, 0xFF, 0xFF, // movq [rsp], -128
            0x49, 0xC7, 0x04, 0x24, 0x00, 0x00, 0x00, 0x00, // movq [r12], 0
            0x49, 0xC7, 0x45, 0x00, 0x63, 0x00, 0x00, 0x00, // movq [r13], 99
            0x49, 0xC7, 0x03, 0x02, 0x00, 0x00, 0x00, // movq [r11], 2
        ]
    )
}

#[test]
fn test_mov_ptr64_imm32_disp8() {
    let mut code = EmitterX64::new();
    code.mov_ptr64_imm32_disp8(RDX, -98, -10)
        .mov_ptr64_imm32_disp8(RBP, 127, 12)
        .mov_ptr64_imm32_disp8(RSP, 2383839, -9)
        .mov_ptr64_imm32_disp8(R12, -129484, 1)
        .mov_ptr64_imm32_disp8(R13, 88, 127)
        .mov_ptr64_imm32_disp8(R8, 0, 16);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0xC7, 0x42, 0xF6, 0x9E, 0xFF, 0xFF, 0xFF, // movq [rdx-10], -98
            0x48, 0xC7, 0x45, 0x0C, 0x7F, 0x00, 0x00, 0x00, // movq [rbp+12], 127
            0x48, 0xC7, 0x44, 0x24, 0xF7, 0xDF, 0x5F, 0x24, 0x00, // movq [rsp-9], 2383839
            0x49, 0xC7, 0x44, 0x24, 0x01, 0x34, 0x06, 0xFE, 0xFF, // movq [r12+1], -129484
            0x49, 0xC7, 0x45, 0x7F, 0x58, 0x00, 0x00, 0x00, // movq [r13+127],88
            0x49, 0xC7, 0x40, 0x10, 0x00, 0x00, 0x00, 0x00, // movq [r8+16], 0
        ]
    )
}

#[test]
fn test_mov_ptr64_reg64_sib_disp32() {
    let mut code = EmitterX64::new();
    code.mov_ptr64_reg64_sib_disp32(RBX, 2, RAX, 128, RCX)
        .mov_ptr64_reg64_sib_disp32(RBP, 4, RBP, -454, RSI)
        .mov_ptr64_reg64_sib_disp32(RSP, 8, R13, 209384, R12)
        .mov_ptr64_reg64_sib_disp32(RDI, 1, R12, -943949, RAX);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0x89, 0x8C, 0x43, 0x80, 0x00, 0x00, 0x00, // mov [rbx+2*rax+128], rcx
            0x48, 0x89, 0xB4, 0xAD, 0x3A, 0xFE, 0xFF, 0xFF, // mov [rbp+4*rbp-454], rsi
            0x4E, 0x89, 0xA4, 0xEC, 0xE8, 0x31, 0x03, 0x00, // mov [rsp+8*r13+209384], r12
            0x4A, 0x89, 0x84, 0x27, 0xB3, 0x98, 0xF1, 0xFF, // mov [rdi+r12-943949],rax
        ]
    )
}

#[test]
fn test_push_reg64_base() {
    let mut code = EmitterX64::new();
    code.push_reg64(RAX)
        .push_reg64(RCX)
        .push_reg64(RDX)
        .push_reg64(RBX)
        .push_reg64(RSP)
        .push_reg64(RBP)
        .push_reg64(RSI)
        .push_reg64(RDI);
    assert_eq!(
        code.buf,
        vec![0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57]
    );
}

#[test]
#[rustfmt::skip]
fn test_push_reg64_extended() {
    let mut code = EmitterX64::new();
    code.push_reg64(R8)
        .push_reg64(R9)
        .push_reg64(R10)
        .push_reg64(R11)
        .push_reg64(R12)
        .push_reg64(R13)
        .push_reg64(R14)
        .push_reg64(R15);
    assert_eq!(code.buf, vec![
        0x41, 0x50, 
        0x41, 0x51, 
        0x41, 0x52,
        0x41, 0x53, 
        0x41, 0x54, 
        0x41, 0x55, 
        0x41, 0x56, 
        0x41, 0x57,
    ]);
}

#[test]
    #[rustfmt::skip]
    fn test_push_ptr64_base() {
        let mut code = EmitterX64::new();
        code.push_ptr64(RAX)
            .push_ptr64(RCX)
            .push_ptr64(RDX)
            .push_ptr64(RBX)
            .push_ptr64(RSP)
            .push_ptr64(RBP)
            .push_ptr64(RSI)
            .push_ptr64(RDI);
        assert_eq!(code.buf, vec![
            0xff, 0x30, 
            0xff, 0x31, 
            0xff, 0x32, 
            0xff, 0x33, 
            0xff, 0x34, 0x24, 
            0xff, 0x75, 0x00, 
            0xff, 0x36, 
            0xff, 0x37,
        ]);
    }

#[test]
#[rustfmt::skip]
fn test_push_ptr64_extended() {
    let mut code = EmitterX64::new();
    code.push_ptr64(R8)
        .push_ptr64(R9)
        .push_ptr64(R10)
        .push_ptr64(R11)
        .push_ptr64(R12)
        .push_ptr64(R13)
        .push_ptr64(R14)
        .push_ptr64(R15);
    assert_eq!(code.buf, vec![
        0x41, 0xff, 0x30, 
        0x41, 0xff, 0x31, 
        0x41, 0xff, 0x32, 
        0x41, 0xff, 0x33, 
        0x41, 0xff, 0x34, 0x24, 
        0x41, 0xff, 0x75, 0x00, 
        0x41, 0xff, 0x36, 
        0x41, 0xff, 0x37,
    ]);
}

#[test]
fn test_push_ptr64_disp8() {
    let mut code = EmitterX64::new();
    code.push_ptr64_disp8(RAX, -39)
        .push_ptr64_disp8(RBP, 88)
        .push_ptr64_disp8(RSP, 99)
        .push_ptr64_disp8(R12, -13)
        .push_ptr64_disp8(R13, 109)
        .push_ptr64_disp8(R15, 2);
    assert_eq!(
        code.buf,
        vec![
            0xFF, 0x70, 0xD9, // push [rax-39]
            0xFF, 0x75, 0x58, // push [rbp+88]
            0xFF, 0x74, 0x24, 0x63, // push [rsp+99]
            0x41, 0xFF, 0x74, 0x24, 0xF3, // push [r12-13]
            0x41, 0xFF, 0x75, 0x6D, // push [r13+109]
            0x41, 0xFF, 0x77, 0x02, // push [r15+2]
        ]
    );
}

#[test]
fn test_pop_reg64_base() {
    let mut code = EmitterX64::new();
    code.pop_reg64(RAX);
    code.pop_reg64(RCX);
    code.pop_reg64(RDX);
    code.pop_reg64(RBX);
    code.pop_reg64(RSP);
    code.pop_reg64(RBP);
    code.pop_reg64(RSI);
    code.pop_reg64(RDI);
    assert_eq!(
        code.buf,
        vec![0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f]
    );
}

#[test]
#[rustfmt::skip]
fn test_pop_reg64_extended() {
    let mut code = EmitterX64::new();
    code.pop_reg64(R8);
    code.pop_reg64(R9);
    code.pop_reg64(R10);
    code.pop_reg64(R11);
    code.pop_reg64(R12);
    code.pop_reg64(R13);
    code.pop_reg64(R14);
    code.pop_reg64(R15);
    assert_eq!(code.buf, vec![
        0x41, 0x58, 
        0x41, 0x59, 
        0x41, 0x5a,
        0x41, 0x5b, 
        0x41, 0x5c, 
        0x41, 0x5d, 
        0x41, 0x5e, 
        0x41, 0x5f,
    ]);
}

#[test]
#[rustfmt::skip]
fn test_pop_ptr64_base() {
    let mut code = EmitterX64::new();
    code.pop_ptr64(RAX);
    code.pop_ptr64(RCX);
    code.pop_ptr64(RDX);
    code.pop_ptr64(RBX);
    code.pop_ptr64(RSP);
    code.pop_ptr64(RBP);
    code.pop_ptr64(RSI);
    code.pop_ptr64(RDI);
    assert_eq!(code.buf, vec![
        0x8f, 0x00, 
        0x8f, 0x01, 
        0x8f, 0x02, 
        0x8f, 0x03, 
        0x8f, 0x04, 0x24, 
        0x8f, 0x45, 0x00, 
        0x8f, 0x06, 
        0x8f, 0x07,
    ]);
}

#[test]
#[rustfmt::skip]
fn test_pop_ptr64_extended() {
    let mut code = EmitterX64::new();
    code.pop_ptr64(R8);
    code.pop_ptr64(R9);
    code.pop_ptr64(R10);
    code.pop_ptr64(R11);
    code.pop_ptr64(R12);
    code.pop_ptr64(R13);
    code.pop_ptr64(R14);
    code.pop_ptr64(R15);
    assert_eq!(code.buf, vec![
        0x41, 0x8f, 0x00, 
        0x41, 0x8f, 0x01, 
        0x41, 0x8f, 0x02, 
        0x41, 0x8f, 0x03, 
        0x41, 0x8f, 0x04, 0x24, 
        0x41, 0x8f, 0x45, 0x00, 
        0x41, 0x8f, 0x06, 
        0x41, 0x8f, 0x07,
    ]);
}

#[test]
fn test_pop_ptr64_disp8() {
    let mut code = EmitterX64::new();
    code.pop_ptr64_disp8(RDX, -39);
    code.pop_ptr64_disp8(RBP, 88);
    code.pop_ptr64_disp8(RSP, 99);
    code.pop_ptr64_disp8(R12, -13);
    code.pop_ptr64_disp8(R13, 109);
    code.pop_ptr64_disp8(R8, 2);
    assert_eq!(
        code.buf,
        vec![
            0x8F, 0x42, 0xD9, // pop [rdx-39]
            0x8F, 0x45, 0x58, // pop [rbp+88]
            0x8F, 0x44, 0x24, 0x63, // pop [rsp+99]
            0x41, 0x8F, 0x44, 0x24, 0xF3, // pop [r12-13]
            0x41, 0x8F, 0x45, 0x6D, // pop [r13+109]
            0x41, 0x8F, 0x40, 0x02, // pop [r8+2]
        ]
    );
}

#[test]
fn test_sub_reg64_imm32() {
    let mut code = EmitterX64::new();
    code.sub_reg64_imm32(RBP, -329).sub_reg64_imm32(RSP, 999);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0x81, 0xED, 0xB7, 0xFE, 0xFF, 0xFF, // sub rbp, -329
            0x48, 0x81, 0xEC, 0xE7, 0x03, 0x00, 0x00, // sub rsp, 999
        ]
    );
}
