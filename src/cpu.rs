pub struct VirtualState {
    pub vregs: [u64; 30],
    pub mem: Box<[u8]>,
}

impl Default for VirtualState {
    fn default() -> VirtualState {
        let mem_size = 4 * (1 << 20);
        VirtualState {
            vregs: [0; 30],
            mem: vec![0; mem_size].into_boxed_slice(),
        }
    }
}
