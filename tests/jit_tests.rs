use ndsjit::{
    ir::parsing::instruction, ir::Instruction, translate::block_translator::BlockTranslator,
};
use std::{mem, ptr};

type Func = unsafe extern "C" fn(*mut [u32; 17]) -> i32;

const PROG: &str = "
    mov r11, #1234
    mov r2, r11
";

#[test]
fn test_MOV() {
    let mut translator = BlockTranslator::new();
    let mut code = vec![];
    for line in PROG.trim().lines() {
        println!("{}", line.trim());
        let (_, instr) = instruction(line.trim()).unwrap();
        dbg!(&instr);
        code.push(instr);
    }
    let func_ptr = translator.translate(&code).unwrap();
    let mut regs = [0u32; 17];
    unsafe {
        let func: Func = mem::transmute(func_ptr);
        func(ptr::addr_of_mut!(regs));
    }
    assert_eq!(regs[2], 1234);
    assert_eq!(regs[11], 1234);
}

fn parse_asm_file(filepath: &str) -> Vec<Instruction> {
    let src = std::fs::read_to_string(filepath).unwrap();
    let mut code = vec![];
    for line in src.trim().lines() {
        let (_, instr) = instruction(line).unwrap();
        code.push(instr);
    }
    code
}

#[test]
fn test_data_proc() {
    let code = parse_asm_file("tests/test_programs/data_proc.asm");
    let mut translator = BlockTranslator::new();
    let func_ptr = translator.translate(&code).unwrap();
    let mut regs = [0u32; 17];
    unsafe {
        let func: Func = mem::transmute(func_ptr);
        func(ptr::addr_of_mut!(regs));
    }
    assert_eq!(regs, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
