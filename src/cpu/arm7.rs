use std::fs;
use std::io::Error;
use std::ptr;

use crate::ir::VReg::PC;

pub enum InstrMode {
    ARM,
    THUMB,
}

pub struct ARM7 {
    // TODO - these should be 32 bit
    pub vregs: [u32; 16],
    pub mem: Box<[u8]>,
    pub instr_mode: InstrMode,
}

/// 225 Mb - includes max space a cartridge can take up
const MEM_SIZE: usize = 0xe100000;

impl ARM7 {
    /// Construct emulated cpu state with all registers and memory set to 0
    pub fn new() -> ARM7 {
        // TODO - a lot of the address space is unused by the GBA. Creating an array covering the
        // the full space is wasteful, but simplifies direct reads. Might try reducing later
        ARM7 {
            vregs: [0; 16],
            mem: vec![0; MEM_SIZE].into_boxed_slice(),
            instr_mode: InstrMode::ARM,
        }
    }

    pub fn load_bios(&mut self, path: &str) -> Result<(), Error> {
        let f = fs::read(path)?;
        for (i, &byte) in f.iter().enumerate() {
            self.mem[i] = byte;
        }
        Ok(())
    }

    pub fn get_pc(&self) -> u32 {
	self.vregs[PC as usize]
    }

    // Obtain a raw pointer to the first byte of the virtual register array
    pub fn vreg_base_ptr(&mut self) -> *mut u8 {
        self.vregs.as_mut_ptr() as *mut u8
    }

    // Obtain a raw pointer to the first byte of the heap-allocated memory array
    pub fn mem_base_ptr(&mut self) -> *mut u8 {
        self.mem.as_mut_ptr() as *mut u8
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        unsafe {
            mem_write_word(self as *mut ARM7, addr, value);
        }
    }

    pub fn read_word(&mut self, addr: u32) -> u32 {
        unsafe { mem_read_word(self as *mut ARM7, addr) }
    }
}

// Callbacks for JIT compiled code. Eventually needed to handle interrupts, MMIO, etc.

pub unsafe extern "C" fn mem_write_word(cpu: *mut ARM7, addr: u32, value: u32) {
    assert!(cpu != ptr::null_mut());
    assert!((addr as usize) < MEM_SIZE);
    for (i, &b) in value.to_le_bytes().iter().enumerate() {
        (*cpu).mem[(addr * 4) as usize + i] = b;
    }
}

pub unsafe extern "C" fn mem_read_word(cpu: *mut ARM7, addr: u32) -> u32 {
    assert!(cpu != ptr::null_mut());
    assert!((addr as usize) < MEM_SIZE);
    let addr = addr as usize;
    let bytes = &(*cpu).mem[addr * 4..addr * 4 + 4];
    u32::from_le_bytes(bytes.try_into().unwrap())
}

// pub unsafe extern "C" fn mem_write_halfword(cpu: *mut ARM7, addr: u32, value: u16) {}
// pub unsafe extern "C" fn mem_write_byte(cpu: *mut ARM7, addr: u32, value: u8) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_raw_vreg_access_1() {
        let mut st = ARM7::new();
        unsafe {
            // mem[0]
            *st.vreg_base_ptr().offset(0) = 0x19;
            *st.vreg_base_ptr().offset(1) = 0x8d;
            *st.vreg_base_ptr().offset(2) = 0x0f;
            *st.vreg_base_ptr().offset(3) = 0x35;
            // Assume system is little-endian
            assert_eq!(st.vregs[0], 0x350f8d19);
        }
    }

    #[test]
    fn test_raw_vreg_access_2() {
        let mut st = ARM7::new();
        unsafe {
            // mem[0]
            let base: isize = 15 * mem::size_of::<u32>() as isize;
            *st.vreg_base_ptr().offset(base + 0) = 0x22;
            *st.vreg_base_ptr().offset(base + 1) = 0xc0;
            *st.vreg_base_ptr().offset(base + 2) = 0x31;
            *st.vreg_base_ptr().offset(base + 3) = 0x9a;
            // Assume system is little-endian
            assert_eq!(st.vregs[15], 0x9a31c022);
        }
    }

    #[test]
    fn test_raw_mem_access_1() {
        let mut st = ARM7::new();
        unsafe {
            *st.mem_base_ptr().offset(97) = 231;
            assert_eq!(st.mem[97], 231);
        }
    }

    #[test]
    fn test_raw_mem_access_2() {
        let mut st = ARM7::new();
        unsafe {
            *st.mem_base_ptr().offset(0) = 12;
            assert_eq!(st.mem[0], 12);
        }
    }

    #[test]
    fn test_read_write_word() {
        let mut st = ARM7::new();
        unsafe {
            let ptr = &mut st as *mut ARM7;
            mem_write_word(ptr, 0, 432284757);
            mem_write_word(ptr, 19, 9989999);
            assert_eq!(mem_read_word(ptr, 0), 432284757);
            assert_eq!(mem_read_word(ptr, 1), 0);
            assert_eq!(mem_read_word(ptr, 19), 9989999);
            assert_eq!(mem_read_word(ptr, 20), 0);
        }
    }
}
