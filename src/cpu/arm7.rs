use std::ptr;

pub struct ARM7 {
    // TODO - these should be 32 bit
    pub vregs: [u32; 16],
    pub mem: Box<[u8]>,
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
        }
    }

    // Obtain a raw pointer to the first byte of the virtual register array
    pub fn vreg_base_ptr(&mut self) -> *mut u8 {
        self.vregs.as_mut_ptr() as *mut u8
    }

    // Obtain a raw pointer to the first byte of the heap-allocated memory array
    pub fn mem_base_ptr(&mut self) -> *mut u8 {
        self.mem.as_mut_ptr() as *mut u8
    }
}

// // Callbacks for JIT compiled code. Eventually needed to handle interrupts, MMIO, etc.

// pub extern "C" fn mem_write(cpu: *mut ARM7, addr: u32, value: u32) {
//     unsafe {
//         assert!(cpu != ptr::null_mut());
//         assert!((addr as usize) < MEM_SIZE);
//         for b in value.to_le_bytes() {
//             (*cpu).mem[addr as usize] = b;
//         }
//     }
// }

// pub extern "C" fn mem_writeh(cpu: *mut ARM7, addr: u32, value: u16) {
//     unsafe {
//         assert!(cpu != ptr::null_mut());
//         assert!((addr as usize) < MEM_SIZE);
//         for b in value.to_le_bytes() {
//             (*cpu).mem[addr as usize] = b;
//         }
//     }
// }

// pub extern "C" fn mem_writeb(cpu: *mut ARM7, addr: u32, value: u8) {
//     unsafe {
//         assert!(cpu != ptr::null_mut());
//         assert!((addr as usize) < MEM_SIZE);
//         (*cpu).mem[addr as usize] = value;
//     }
// }

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
}
