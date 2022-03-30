use super::*;

#[cfg(test)]
mod mov {
    use super::*;

    #[test]
    fn test_mov_reg64_ptr64_1() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R8, RegX64::RBP);
        assert_eq!(code, vec![0x4C, 0x8B, 0x45, 0x00]); // mov r8 [rbp]
    }
    #[test]
    fn test_mov_reg64_ptr64_3() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R15, RegX64::RSI);
        assert_eq!(code, vec![0x4C, 0x8B, 0x3E]); // mov r15, [rsi]
    }
    #[test]
    fn test_mov_reg64_ptr64_4() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RDI, RegX64::RBX);
        assert_eq!(code, vec![0x48, 0x8B, 0x3B]); // mov rdi,[rbx]
    }
    #[test]
    fn test_mov_reg64_ptr64_5() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RAX, RegX64::RAX);
        assert_eq!(code, vec![0x48, 0x8B, 0x00]); // mov rax,[rax]
    }
    #[test]
    fn test_mov_reg64_ptr64_6() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R11, RegX64::RCX);
        assert_eq!(code, vec![0x4C, 0x8B, 0x19]); // mov r11,[rcx]
    }
    #[test]
    fn test_mov_reg64_ptr64_7() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RBP, RegX64::RSP);
        assert_eq!(code, vec![0x48, 0x8B, 0x2C, 0x24]); //mov rbp,[rsp]
    }
    #[test]
    fn test_mov_reg64_ptr64_8() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RCX, RegX64::RDI);
        assert_eq!(code, vec![0x48, 0x8B, 0x0F]); // mov rcx,[rdi]
    }
    #[test]
    fn test_mov_reg64_ptr64_9() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::R9, RegX64::R12);
        assert_eq!(code, vec![0x4D, 0x8B, 0x0C, 0x24]) // mov r9,[r12]
    }
    #[test]
    fn test_mov_reg64_ptr64_10() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64(&mut code, RegX64::RAX, RegX64::R13);
        assert_eq!(code, vec![0x49, 0x8B, 0x45, 0x00]) // mov rax,[r13]
    }

    #[test]
    fn test_mov_ptr64_reg64_1() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::RBP, RegX64::RDI);
        assert_eq!(code, vec![0x48, 0x89, 0x7D, 0x00]) // mov [rbp],rdi
    }
    #[test]
    fn test_mov_ptr64_reg64_2() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::RSP, RegX64::RAX);
        assert_eq!(code, vec![0x48, 0x89, 0x04, 0x24]) // mov [rsp],rax
    }
    #[test]
    fn test_mov_ptr64_reg64_3() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::R12, RegX64::R15);
        assert_eq!(code, vec![0x4D, 0x89, 0x3C, 0x24]) // mov [r12],r15
    }
    #[test]
    fn test_mov_ptr64_reg64_4() {
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64(&mut code, RegX64::R13, RegX64::R13);
        assert_eq!(code, vec![0x4D, 0x89, 0x6D, 0x00]) // mov r13,[r13]
    }

    #[test]
    fn test_mov_reg64_reg64_1() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_reg64(&mut code, RegX64::RBX, RegX64::RDX);
        assert_eq!(code, vec![0x48, 0x89, 0xD3]) // mov rbx,rdx
    }
    #[test]
    fn test_mov_reg64_reg64_2() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_reg64(&mut code, RegX64::RDX, RegX64::RBP);
        assert_eq!(code, vec![0x48, 0x89, 0xEA]) // mov rdx,rbp
    }
    #[test]
    fn test_mov_reg64_reg64_3() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_reg64(&mut code, RegX64::R9, RegX64::RSP);
        assert_eq!(code, vec![0x49, 0x89, 0xE1]) // mov r9,rsp
    }
    #[test]
    fn test_mov_reg64_reg64_4() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_reg64(&mut code, RegX64::RCX, RegX64::R12);
        assert_eq!(code, vec![0x4C, 0x89, 0xE1]) // mov rcx,r12
    }

    #[test]
    fn test_mov_reg64_ptr64_sib() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64_sib(&mut code, RegX64::RAX, RegX64::RSP, RegX64::R12, 8);
        mov_reg64_ptr64_sib(&mut code, RegX64::RSP, RegX64::RBP, RegX64::RBP, 1);
        mov_reg64_ptr64_sib(&mut code, RegX64::RDI, RegX64::R12, RegX64::R13, 4);
        mov_reg64_ptr64_sib(&mut code, RegX64::RBX, RegX64::RDX, RegX64::RBP, 2);
        mov_reg64_ptr64_sib(&mut code, RegX64::R13, RegX64::R13, RegX64::RAX, 1);
        assert_eq!(
            code,
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
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64_sib(&mut code, RegX64::RAX, RegX64::RAX, RegX64::RSP, 1);
    }

    #[test]
    fn test_mov_reg64_ptr64_disp8() {
        let mut code: Vec<u8> = Vec::new();
        mov_reg64_ptr64_disp8(&mut code, RegX64::R8, RegX64::RBP, 127);
        mov_reg64_ptr64_disp8(&mut code, RegX64::R9, RegX64::RSP, -127);
        mov_reg64_ptr64_disp8(&mut code, RegX64::R10, RegX64::R12, 99);
        mov_reg64_ptr64_disp8(&mut code, RegX64::R11, RegX64::R13, -45);
        mov_reg64_ptr64_disp8(&mut code, RegX64::RCX, RegX64::R15, 109);
        mov_reg64_ptr64_disp8(&mut code, RegX64::RBX, RegX64::RAX, 12);
        assert_eq!(
            code,
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
        let mut code: Vec<u8> = Vec::new();
        mov_ptr64_reg64_disp8(&mut code, RegX64::RBP, RegX64::RAX, -78);
        mov_ptr64_reg64_disp8(&mut code, RegX64::RSP, RegX64::RBX, 10);
        mov_ptr64_reg64_disp8(&mut code, RegX64::R12, RegX64::RCX, -3);
        mov_ptr64_reg64_disp8(&mut code, RegX64::R13, RegX64::R15, 44);
        mov_ptr64_reg64_disp8(&mut code, RegX64::RDI, RegX64::RSI, -1);
        assert_eq!(
            code,
            vec![
                0x48, 0x89, 0x45, 0xB2, // mov [rbp-78], rax
                0x48, 0x89, 0x5C, 0x24, 0x0A, // mov [rsp+10], rbx
                0x49, 0x89, 0x4C, 0x24, 0xFD, // mov [r12-3], rcx
                0x4D, 0x89, 0x7D, 0x2C, // mov [r13+44],r15
                0x48, 0x89, 0x77, 0xFF, // mov [rdi-1], rsi
            ]
        )
    }
}

#[cfg(test)]
mod push {
    use super::*;
    #[test]
    fn test_push_reg64_base() {
        let mut code: Vec<u8> = Vec::new();
        push_reg64(&mut code, RegX64::RAX);
        push_reg64(&mut code, RegX64::RCX);
        push_reg64(&mut code, RegX64::RDX);
        push_reg64(&mut code, RegX64::RBX);
        push_reg64(&mut code, RegX64::RSP);
        push_reg64(&mut code, RegX64::RBP);
        push_reg64(&mut code, RegX64::RSI);
        push_reg64(&mut code, RegX64::RDI);
        assert_eq!(code, vec![0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57]);
    }
    #[test]
    #[rustfmt::skip]
    fn test_push_reg64_extended() {
        let mut code: Vec<u8> = Vec::new();
        push_reg64(&mut code, RegX64::R8);
        push_reg64(&mut code, RegX64::R9);
        push_reg64(&mut code, RegX64::R10);
        push_reg64(&mut code, RegX64::R11);
        push_reg64(&mut code, RegX64::R12);
        push_reg64(&mut code, RegX64::R13);
        push_reg64(&mut code, RegX64::R14);
        push_reg64(&mut code, RegX64::R15);
        assert_eq!(code, vec![
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
        let mut code: Vec<u8> = Vec::new();
        push_ptr64(&mut code, RegX64::RAX);
        push_ptr64(&mut code, RegX64::RCX);
        push_ptr64(&mut code, RegX64::RDX);
        push_ptr64(&mut code, RegX64::RBX);
        push_ptr64(&mut code, RegX64::RSP);
        push_ptr64(&mut code, RegX64::RBP);
        push_ptr64(&mut code, RegX64::RSI);
        push_ptr64(&mut code, RegX64::RDI);
        assert_eq!(code, vec![
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
        let mut code: Vec<u8> = Vec::new();
        push_ptr64(&mut code, RegX64::R8);
        push_ptr64(&mut code, RegX64::R9);
        push_ptr64(&mut code, RegX64::R10);
        push_ptr64(&mut code, RegX64::R11);
        push_ptr64(&mut code, RegX64::R12);
        push_ptr64(&mut code, RegX64::R13);
        push_ptr64(&mut code, RegX64::R14);
        push_ptr64(&mut code, RegX64::R15);
        assert_eq!(code, vec![
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
        let mut code: Vec<u8> = Vec::new();
        push_ptr64_disp8(&mut code, RegX64::RAX, -39);
        push_ptr64_disp8(&mut code, RegX64::RBP, 88);
        push_ptr64_disp8(&mut code, RegX64::RSP, 99);
        push_ptr64_disp8(&mut code, RegX64::R12, -13);
        push_ptr64_disp8(&mut code, RegX64::R13, 109);
        push_ptr64_disp8(&mut code, RegX64::R15, 2);
        assert_eq!(
            code,
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
}

#[cfg(test)]
mod pop {
    use super::*;
    #[test]
    fn test_pop_reg64_base() {
        let mut code: Vec<u8> = Vec::new();
        pop_reg64(&mut code, RegX64::RAX);
        pop_reg64(&mut code, RegX64::RCX);
        pop_reg64(&mut code, RegX64::RDX);
        pop_reg64(&mut code, RegX64::RBX);
        pop_reg64(&mut code, RegX64::RSP);
        pop_reg64(&mut code, RegX64::RBP);
        pop_reg64(&mut code, RegX64::RSI);
        pop_reg64(&mut code, RegX64::RDI);
        assert_eq!(code, vec![0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f]);
    }
    #[test]
    #[rustfmt::skip]
    fn test_pop_reg64_extended() {
        let mut code: Vec<u8> = Vec::new();
        pop_reg64(&mut code, RegX64::R8);
        pop_reg64(&mut code, RegX64::R9);
        pop_reg64(&mut code, RegX64::R10);
        pop_reg64(&mut code, RegX64::R11);
        pop_reg64(&mut code, RegX64::R12);
        pop_reg64(&mut code, RegX64::R13);
        pop_reg64(&mut code, RegX64::R14);
        pop_reg64(&mut code, RegX64::R15);
        assert_eq!(code, vec![
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
        let mut code: Vec<u8> = Vec::new();
        pop_ptr64(&mut code, RegX64::RAX);
        pop_ptr64(&mut code, RegX64::RCX);
        pop_ptr64(&mut code, RegX64::RDX);
        pop_ptr64(&mut code, RegX64::RBX);
        pop_ptr64(&mut code, RegX64::RSP);
        pop_ptr64(&mut code, RegX64::RBP);
        pop_ptr64(&mut code, RegX64::RSI);
        pop_ptr64(&mut code, RegX64::RDI);
        assert_eq!(code, vec![
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
        let mut code: Vec<u8> = Vec::new();
        pop_ptr64(&mut code, RegX64::R8);
        pop_ptr64(&mut code, RegX64::R9);
        pop_ptr64(&mut code, RegX64::R10);
        pop_ptr64(&mut code, RegX64::R11);
        pop_ptr64(&mut code, RegX64::R12);
        pop_ptr64(&mut code, RegX64::R13);
        pop_ptr64(&mut code, RegX64::R14);
        pop_ptr64(&mut code, RegX64::R15);
        assert_eq!(code, vec![
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
        let mut code: Vec<u8> = Vec::new();
        pop_ptr64_disp8(&mut code, RegX64::RDX, -39);
        pop_ptr64_disp8(&mut code, RegX64::RBP, 88);
        pop_ptr64_disp8(&mut code, RegX64::RSP, 99);
        pop_ptr64_disp8(&mut code, RegX64::R12, -13);
        pop_ptr64_disp8(&mut code, RegX64::R13, 109);
        pop_ptr64_disp8(&mut code, RegX64::R8, 2);
        assert_eq!(
            code,
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
}
