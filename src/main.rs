use std::{error::Error, mem, ptr};

use ndsjit::asm::block_translator::BlockTranslator;

#[repr(C, packed)]
#[derive(Default, Copy, Clone, Debug)]
struct Regs {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut translator = BlockTranslator::new();
    let code = vec![];
    let func_ptr = translator.translate(&code)?;

    // let mut vm_state = Regs::default();
    let mut vm_state = [0u32; 17];
    vm_state[16] = 1;
    dbg!(vm_state);

    unsafe {
        let func: unsafe extern "C" fn(*mut [u32; 17]) -> i32 = mem::transmute(func_ptr);
        dbg!(func(ptr::addr_of_mut!(vm_state)));
    }
    dbg!(vm_state);

    Ok(())
}
