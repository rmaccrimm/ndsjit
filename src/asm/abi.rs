use crate::ir::VReg;
use std::mem;
use std::vec::Vec;

pub const TEMP_REG: RegX64 = EAX;
#[cfg(target_os="windows")]
pub const VREG_ADDR_REG: RegX64 = ECX;
#[cfg(target_os="linux")]
pub const VREG_ADDR_REG: RegX64 = EDI;

#[cfg(target_os="windows")]
pub const VMEM_ADDR_REG: RegX64 = EDX;
#[cfg(target_os="linux")]
pub const VMEM_ADDR_REG: RegX64 = ESI;

// Does not include RBP and RSP (which are also saved/restored)
#[cfg(target_os="windows")]
pub const CALLEE_SAVED_REGS: [RegX64; 7] = [RBX, RDI, RSI, R12, R13, R15, R15];
#[cfg(target_os="linux")]
pub const CALLEE_SAVED_REGS: [RegX64; 5] = [RBX, R12, R13, R14, R15];
