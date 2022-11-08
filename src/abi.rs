use crate::compiler::emitter::{
    RegX64, EAX, ECX, EDI, EDX, ESI, R12, R13, R14, R15, RBX, RDI, RSI,
};

pub const TEMP_REG: RegX64 = EAX;

// Todo - change this to an array of argument-passing registers
#[cfg(target_os = "windows")]
pub const VREG_ADDR_REG: RegX64 = ECX;
#[cfg(target_os = "linux")]
pub const VREG_ADDR_REG: RegX64 = EDI;

#[cfg(target_os = "windows")]
pub const VMEM_ADDR_REG: RegX64 = EDX;
#[cfg(target_os = "linux")]
pub const VMEM_ADDR_REG: RegX64 = ESI;

// Does not include RBP and RSP (which are also saved/restored)
#[cfg(target_os = "windows")]
pub const CALLEE_SAVED_REGS: [RegX64; 7] = [RBX, RDI, RSI, R12, R13, R14, R15];
#[cfg(target_os = "linux")]
pub const CALLEE_SAVED_REGS: [RegX64; 5] = [RBX, R12, R13, R14, R15];
