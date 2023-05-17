use std::mem::size_of;

use crate::ir::Register;

#[repr(C, packed)]
pub struct VMState {
    regs: [u32; 17],
}

impl VMState {}
