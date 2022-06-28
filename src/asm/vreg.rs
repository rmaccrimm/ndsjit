use super::x64::{Address, EmitterX64, RegX64};

pub mod constants {}
use constants::*;

pub struct VReg {
    pub virt_loc: Address,
    pub mapped_reg: Option<RegX64>,
}

impl VReg {
    pub fn new(virt_loc: Address, mapped_reg: Option<RegX64>) -> VReg {
        VReg {
            virt_loc,
            mapped_reg,
        }
    }
}

#[cfg(test)]
mod tests {}
