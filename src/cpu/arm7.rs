pub struct ARM7 {
    pub vregs: [u64; 16],
    pub mem: Box<[u8]>,
}

impl ARM7 {
    // Construct emulated cpu state with all registers and memory set to 0
    pub fn new() -> ARM7 {
        let mem_size = 4 * (1 << 20);
        ARM7 {
            vregs: [0; 16],
            mem: vec![0; mem_size].into_boxed_slice(),
        }
    }

    // Obtain a raw pointer to the first byte of the virtual register array
    pub fn vreg_base_ptr(&mut self) -> *mut u8 {
        self.vregs.as_mut_ptr() as *mut u8
    }

    // Obtain a raw pointer to the first byte of the heap-allocated memory array
    pub fn mem_base_ptr(&mut self) -> *mut u8 {
        (*self.mem).as_mut_ptr() as *mut u8
    }
}

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
            *st.vreg_base_ptr().offset(4) = 0x44;
            *st.vreg_base_ptr().offset(5) = 0x00;
            *st.vreg_base_ptr().offset(6) = 0xbc;
            *st.vreg_base_ptr().offset(7) = 0x9e;
            // Assume system is little-endian
            assert_eq!(st.vregs[0], 0x9ebc0044350f8d19);
        }
    }
    #[test]
    fn test_raw_vreg_access_2() {
        let mut st = ARM7::new();
        unsafe {
            // mem[0]
            let base: isize = 15 * mem::size_of::<u64>() as isize;
            *st.vreg_base_ptr().offset(base + 0) = 0x22;
            *st.vreg_base_ptr().offset(base + 1) = 0xc0;
            *st.vreg_base_ptr().offset(base + 2) = 0x31;
            *st.vreg_base_ptr().offset(base + 3) = 0x9a;
            *st.vreg_base_ptr().offset(base + 4) = 0x4f;
            *st.vreg_base_ptr().offset(base + 5) = 0x66;
            *st.vreg_base_ptr().offset(base + 6) = 0x01;
            *st.vreg_base_ptr().offset(base + 7) = 0xf0;
            // Assume system is little-endian
            assert_eq!(st.vregs[15], 0xf001664f9a31c022);
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
