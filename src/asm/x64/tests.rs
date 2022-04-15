use super::RegX64::*;
use super::*;

#[test]
fn test_add_reg32_reg32() {
    let mut code = EmitterX64::new();
    code.add_reg32_reg32(RBX, RBP)
        .add_reg32_reg32(RAX, R15)
        .add_reg32_reg32(R11, RSI)
        .add_reg32_reg32(R8, R9);
    assert_eq!(
        code.buf,
        vec![
            0x01, 0xEB, // add ebx, ebp
            0x44, 0x01, 0xF8, // add eax, r15d
            0x41, 0x01, 0xF3, // add r11d, esi
            0x45, 0x01, 0xC8, // add r8d, r9d
        ]
    );
}

#[test]
fn test_add_reg32_ptr64_disp8() {
    let mut code = EmitterX64::new();
    code.add_reg32_ptr64_disp8(RBX, RBP, 12)
        .add_reg32_ptr64_disp8(RAX, R12, -128)
        .add_reg32_ptr64_disp8(R11, RSP, 90)
        .add_reg32_ptr64_disp8(R8, R13, 127)
        .add_reg32_ptr64_disp8(RDX, RCX, 70);
    assert_eq!(
        code.buf,
        vec![
            0x03, 0x5D, 0x0C, // add ebx, [rbp+12]
            0x41, 0x03, 0x44, 0x24, 0x80, // add eax, [r12-128]
            0x44, 0x03, 0x5C, 0x24, 0x5A, // add r11d, [rsp+90]
            0x45, 0x03, 0x45, 0x7F, // add r8d,[r13+127]
            0x03, 0x51, 0x46, // add edx, [rcx+70]
        ]
    );
}

#[test]
fn test_mov_reg32_reg32() {
    let mut code = EmitterX64::new();
    code.mov_reg_reg(Reg32(RAX), Reg32(R15))
        .mov_reg_reg(Reg32(RSP), Reg32(RBP))
        .mov_reg_reg(Reg32(RBX), Reg32(R9));
    assert_eq!(
        code.buf,
        vec![
            0x44, 0x89, 0xF8, // mov eax, r15d
            0x89, 0xEC, // mov esp, ebp
            0x44, 0x89, 0xCB, // mov ebx, r9d
        ]
    );
}

#[test]
fn test_mov_reg64_reg64() {
    let mut code = EmitterX64::new();
    code.mov_reg_reg(Reg64(RBX), Reg64(RDX))
        .mov_reg_reg(Reg64(RDX), Reg64(RBP))
        .mov_reg_reg(Reg64(R9), Reg64(RSP))
        .mov_reg_reg(Reg64(RCX), Reg64(R12));
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
#[should_panic]
fn test_mov_reg_different_sizes() {
    let mut code = EmitterX64::new();
    code.mov_reg_reg(Reg32(RAX), Reg64(R12));
}

#[test]
fn test_mov_reg32_ptr64() {
    let mut code = EmitterX64::new();
    code.mov_reg_ptr(Reg32(R8), BaseNoDisp { base: RBP })
        .mov_reg_ptr(Reg32(R15), BaseNoDisp { base: RSI })
        .mov_reg_ptr(Reg32(RDI), BaseNoDisp { base: RBX })
        .mov_reg_ptr(Reg32(RAX), BaseNoDisp { base: RAX })
        .mov_reg_ptr(Reg32(R11), BaseNoDisp { base: RCX })
        .mov_reg_ptr(Reg32(RBP), BaseNoDisp { base: RSP })
        .mov_reg_ptr(Reg32(RCX), BaseNoDisp { base: RDI })
        .mov_reg_ptr(Reg32(R9), BaseNoDisp { base: R12 })
        .mov_reg_ptr(Reg32(RAX), BaseNoDisp { base: R13 });
    assert_eq!(
        code.buf,
        vec![
            0x44, 0x8B, 0x45, 0x00, // mov r8d, [rbp]
            0x44, 0x8B, 0x3E, // mov r15d, [rsi]
            0x8B, 0x3B, // mov edi, [rbx]
            0x8B, 0x00, // mov eax, [rax]
            0x44, 0x8B, 0x19, // mov r11d, [rcx]
            0x8B, 0x2C, 0x24, // mov ebp, [rsp]
            0x8B, 0x0F, // mov ecx, [rdi]
            0x45, 0x8B, 0x0C, 0x24, // mov r9d, [r12]
            0x41, 0x8B, 0x45, 0x00, // mov eax, [r13]
        ]
    );
}

#[test]
fn test_mov_ptr64_reg32() {
    let mut code = EmitterX64::new();
    code.mov_ptr_reg(BaseNoDisp { base: RBP }, Reg32(RDI))
        .mov_ptr_reg(BaseNoDisp { base: RSP }, Reg32(RAX))
        .mov_ptr_reg(BaseNoDisp { base: R12 }, Reg32(R15))
        .mov_ptr_reg(BaseNoDisp { base: R13 }, Reg32(R13));
    assert_eq!(
        code.buf,
        vec![
            0x89, 0x7D, 0x00, // mov [rbp], edi
            0x89, 0x04, 0x24, // mov [rsp], eax
            0x45, 0x89, 0x3C, 0x24, // mov [r12], r15d
            0x45, 0x89, 0x6D, 0x00, // mov [r13], r13d
        ]
    )
}

#[test]
#[rustfmt::skip]
fn test_mov_reg32_ptr64_disp8() {
    let mut code = EmitterX64::new();
    code.mov_reg_ptr(Reg32(R8), BaseDisp8{base: RBP, disp: 127})
        .mov_reg_ptr(Reg32(R9), BaseDisp8{base: RSP, disp: 10})
        .mov_reg_ptr(Reg32(R10), BaseDisp8{base: R12, disp: 99})
        .mov_reg_ptr(Reg32(R11), BaseDisp8{base: R13, disp: -45})
        .mov_reg_ptr(Reg32(RCX), BaseDisp8{base: R15, disp: 109})
        .mov_reg_ptr(Reg32(RBX), BaseDisp8{base: RAX, disp: 12});
    assert_eq!(
        code.buf,
        vec![
            0x44, 0x8B, 0x45, 0x7F, // mov r8d,[rbp+127]
            0x44, 0x8B, 0x4C, 0x24, 0x0A, // mov r9d, [rsp+10]
            0x45, 0x8B, 0x54, 0x24, 0x63, // mov r10d,[r12+99]
            0x45, 0x8B, 0x5D, 0xD3, // mov r11d,[r13-45]
            0x41, 0x8B, 0x4F, 0x6D, // mov ecx,[r15+109]
            0x8B, 0x58, 0x0C, // mov ebx,[rax+12]
        ]
    )
}

#[test]
#[rustfmt::skip]
fn test_mov_ptr64_reg32_disp8() {
    let mut code = EmitterX64::new();
    code.mov_ptr_reg(BaseDisp8{base: RBP, disp: -78}, Reg32(RAX))
        .mov_ptr_reg(BaseDisp8{base: RSP, disp: 10}, Reg32(RBX))
        .mov_ptr_reg(BaseDisp8{base: R12, disp: -3}, Reg32(RCX))
        .mov_ptr_reg(BaseDisp8{base: R13, disp: 44}, Reg32(R15))
        .mov_ptr_reg(BaseDisp8{base: RDI, disp: -1}, Reg32(RSI));
    assert_eq!(
        code.buf,
        vec![
            0x89, 0x45, 0xB2, // mov [rbp-78], eax
            0x89, 0x5C, 0x24, 0x0A, // mov [rsp+10], ebx
            0x41, 0x89, 0x4C, 0x24, 0xFD, // mov [r12-3], ecx
            0x45, 0x89, 0x7D, 0x2C, // mov [r13+44],r15d
            0x89, 0x77, 0xFF, // mov [rdi-1], esi
        ]
    )
}

#[test]
#[rustfmt::skip]
fn test_mov_reg32_ptr64_disp32() {
    let mut code = EmitterX64::new();
    code.mov_reg_ptr(Reg32(RBX), BaseDisp32{base: RSP, disp: 16000})
        .mov_reg_ptr(Reg32(RSP), BaseDisp32{base: RBP, disp: 453})
        .mov_reg_ptr(Reg32(R14), BaseDisp32{base: R12, disp: -883})
        .mov_reg_ptr(Reg32(RSI), BaseDisp32{base: R13, disp: -10000});
    assert_eq!(
        code.buf,
        vec![
            0x8B, 0x9C, 0x24, 0x80, 0x3E, 0x00, 0x00, // mov ebx, [rsp+16000]
            0x8B, 0xA5, 0xC5, 0x01, 0x00, 0x00, // mov esp, [rbp+453]
            0x45, 0x8B, 0xB4, 0x24, 0x8D, 0xFC, 0xFF, 0xFF, // mov r14d, [r12-883]
            0x41, 0x8B, 0xB5, 0xF0, 0xD8, 0xFF, 0xFF, // mov esi, [r13-10000]
        ]
    );
}

#[test]
#[rustfmt::skip]
fn test_mov_ptr64_reg32_disp32() {
    let mut code = EmitterX64::new();
    code.mov_ptr_reg(BaseDisp32{base: RSP, disp: 16000}, Reg32(R11))
        .mov_ptr_reg(BaseDisp32{base: RBP, disp: 453}, Reg32(RAX))
        .mov_ptr_reg(BaseDisp32{base: R12, disp: -883}, Reg32(RDI))
        .mov_ptr_reg(BaseDisp32{base: R13, disp: -10000}, Reg32(RCX));
    assert_eq!(
        code.buf,
        vec![
            0x44, 0x89, 0x9C, 0x24, 0x80, 0x3E, 0x00, 0x00, // mov [rsp+16000], r11d
            0x89, 0x85, 0xC5, 0x01, 0x00, 0x00, // mov [rbp+453], eax
            0x41, 0x89, 0xBC, 0x24, 0x8D, 0xFC, 0xFF, 0xFF, // mov [r12-883], edi
            0x41, 0x89, 0x8D, 0xF0, 0xD8, 0xFF, 0xFF, // mov [r13-10000], ecx
        ]
    );
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
#[rustfmt::skip]
fn test_mov_reg32_ptr64_sib_disp32() {
    let mut code = EmitterX64::new();
    code.mov_reg_ptr(Reg32(RCX), SIBDisp32{base: RBX, scale: 2, index: RAX, disp: 128})
        .mov_reg_ptr(Reg32(RSI), SIBDisp32{base: RBP, scale: 4, index: RBP, disp: -454})
        .mov_reg_ptr(Reg32(R12), SIBDisp32{base: RSP, scale: 8, index: R13, disp: 209384})
        .mov_reg_ptr(Reg32(RAX), SIBDisp32{base: RDI, scale: 1, index: R12, disp: -943949})
        .mov_reg_ptr(Reg32(RSP), SIBDisp32{base: R13, scale: 1, index: R8, disp: -129})
        .mov_reg_ptr(Reg32(R15), SIBDisp32{base: R12, scale: 1, index: RBX, disp: 349999});
    assert_eq!(
        code.buf,
        vec![
            0x8B, 0x8C, 0x43, 0x80, 0x00, 0x00, 0x00, // mov ecx, [rbx+2*rax+128]
            0x8B, 0xB4, 0xAD, 0x3A, 0xFE, 0xFF, 0xFF, // mov esi, [rbp+4*rbp-454]
            0x46, 0x8B, 0xA4, 0xEC, 0xE8, 0x31, 0x03, 0x00, // mov r12d, [rsp+8*r13+209384]
            0x42, 0x8B, 0x84, 0x27, 0xB3, 0x98, 0xF1, 0xFF, // mov eax, [rdi+r12-943949]
            0x43, 0x8B, 0xA4, 0x05, 0x7F, 0xFF, 0xFF, 0xFF, // mov esp, [r13+r8-129]
            0x45, 0x8B, 0xBC, 0x1C, 0x2F, 0x57, 0x05, 0x00, // mov r15d, [r12+rbx+349999]
        ]
    );
}

#[test]
#[should_panic]
#[rustfmt::skip]
fn test_mov_reg32_ptr64_sib_disp32_sp_index() {
    let mut code = EmitterX64::new();
    code.mov_reg_ptr(Reg32(RCX), SIBDisp32{base: RBX, scale: 2, index: RSP, disp: 128});
}

#[test]
#[rustfmt::skip]
fn test_mov_reg32_ptr64_sib() {
    let mut code = EmitterX64::new();
    code.mov_reg_ptr(Reg32(RCX), SIBNoDisp{base: RBX, scale: 2, index: RAX})
        .mov_reg_ptr(Reg32(RSI), SIBNoDisp{base: RBP, scale: 4, index: RBP})
        .mov_reg_ptr(Reg32(R12), SIBNoDisp{base: RSP, scale: 8, index: R13})
        .mov_reg_ptr(Reg32(RAX), SIBNoDisp{base: RDI, scale: 1, index: R12})
        .mov_reg_ptr(Reg32(RSP), SIBNoDisp{base: R13, scale: 1, index: R8})
        .mov_reg_ptr(Reg32(R15), SIBNoDisp{base: R12, scale: 1, index: RBX});
    assert_eq!(
        code.buf,
        vec![
            0x8B, 0x0C, 0x43, // mov ecx, [rbx+2*rax]
            0x8B, 0x74, 0xAD, 0x00, // mov esi, [rbp+4*rbp]
            0x46, 0x8B, 0x24, 0xEC, // mov r12d, [rsp+8*r13]
            0x42, 0x8B, 0x04, 0x27, // mov eax, [rdi+r12]
            0x43, 0x8B, 0x64, 0x05, 0x00, // mov esp, [r13+r8]
            0x45, 0x8B, 0x3C, 0x1C, // mov r15d, [r12+rbx]
        ]
    );
}

#[test]
fn test_mov_reg64_imm64() {
    let mut code = EmitterX64::new();
    code.mov_reg64_imm64(RAX, 500000000000);
    assert_eq!(
        code.buf,
        vec![
            0x48, 0xB8, 0x00, 0x88, 0x52, 0x6A, 0x74, 0x00, 0x00,
            0x00 // mov rax, 500000000000
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
