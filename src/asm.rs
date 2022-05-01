pub mod alloc;
mod execbuffer;
mod x64;
mod x64_ref;
use std::mem;

use super::ir::{Instr, Opcode::*, VReg};
use alloc::{MappedReg::*, RegAllocation};
use execbuffer::ExecBuffer;

use x64::*;
use x64::{PtrOperand::*, RegOperand::*, RegX64::*};

pub struct AssemblerX64 {
    code: EmitterX64,
    reg_alloc: RegAllocation,
}

const REG_SIZE: usize = mem::size_of::<u32>() as usize;

// Stack contains prev stack pointer, followed by spilled registers
const SPILL_START: isize = -1 * mem::size_of::<u64>() as isize;

fn spill_stack_disp(ind: usize) -> isize {
    SPILL_START - ((REG_SIZE * ind) as isize)
}

/*
Planned use - these methods won't be called directly to setup machine code, but instead the
emit/translate/assemble (tbd) function will be passed IR instructions to encode, and then the
get_exec_buffer will be called, something like:

    let func = AssemblerX64::new(reg_alloc)
        .emit(&instructions)
        .get_exec_buffer();
*/
impl AssemblerX64 {
    pub fn new(reg_alloc: RegAllocation) -> AssemblerX64 {
        AssemblerX64 {
            code: EmitterX64::new(),
            reg_alloc,
        }
    }

    pub fn get_exec_buffer(self) -> ExecBuffer {
        ExecBuffer::from_vec(self.code.buf).unwrap()
    }

    pub fn hex_dump(&self) {
        for b in self.code.buf.iter() {
            print!("{:02x}", b);
        }
        println!();
    }

    // Initialize physical register values with those in virtual registers (looked up through
    // pointer in %rcx) and set up the stack. If we have to spill to memory, I guess that will make
    // use of the (physical) stack?
    pub fn gen_prologue(&mut self) -> &mut Self {
        let mut stack_size = self.reg_alloc.num_spilled as i32 * mem::size_of::<u64>() as i32;
        if stack_size % 16 == 0 {
            // Ensures 16-byte stack allignment after call instructions (pushes 8 byte ret addr)
            stack_size += 8;
        }
        self.code
            .push_reg64(RBP)
            .mov_reg_reg(Reg64(RBP), Reg64(RSP))
            .sub_reg64_imm32(RSP, stack_size);

        for (i, mapping) in self.reg_alloc.mapping.iter().enumerate() {
            let vreg_disp = REG_SIZE * i;
            assert!(vreg_disp < (1 << 7));

            match mapping {
                Phys(r) => {
                    self.code.mov_reg_ptr(
                        Reg32(*r),
                        BaseDisp8 {
                            base: RCX,
                            disp: vreg_disp as i8,
                        },
                    );
                }
                Spill(ri) => {
                    // Since prev base ptr is first on stack, add 1 to each index
                    self.code
                        .mov_reg_ptr(
                            Reg32(RAX),
                            BaseDisp8 {
                                base: RCX,
                                disp: vreg_disp as i8,
                            },
                        )
                        .mov_ptr_reg(
                            BaseDisp8 {
                                base: RBP,
                                disp: spill_stack_disp(*ri) as i8,
                            },
                            Reg32(RAX),
                        );
                }
                Unmapped => (),
            };
        }
        self
    }

    /// Move physical register values back to virtual state (through pointer still stored in rcx)
    pub fn gen_epilogue(&mut self) -> &mut Self {
        for (i, mapping) in self.reg_alloc.mapping.iter().enumerate() {
            let vreg_disp = mem::size_of::<u32>() * i;
            assert!(vreg_disp < (1 << 7));
            match mapping {
                Phys(r) => {
                    self.code.mov_ptr_reg(
                        BaseDisp8 {
                            base: RCX,
                            disp: vreg_disp as i8,
                        },
                        Reg32(*r),
                    );
                }
                Spill(i) => {
                    self.code
                        .mov_reg_ptr(
                            Reg32(RAX),
                            BaseDisp8 {
                                base: RBP,
                                disp: spill_stack_disp(*i) as i8,
                            },
                        )
                        .mov_ptr_reg(
                            BaseDisp8 {
                                base: RCX,
                                disp: vreg_disp as i8,
                            },
                            Reg32(RAX),
                        );
                }
                _ => (),
            }
        }
        self.code
            .mov_reg_reg(Reg64(RSP), Reg64(RBP))
            .pop_reg64(RBP)
            .ret();
        self
    }

    pub fn emit(&mut self, instr: Instr) {
        // match instr.opcode {
        //     MOVr(dest, src) => self.mov_reg(dest, src),
        //     MOVi(dest, imm) => self.mov_imm(dest, imm),
        // };
    }

    fn mov_reg(&mut self, dest: VReg, src: VReg) -> &mut Self {
        match (self.reg_alloc.get(dest), self.reg_alloc.get(src)) {
            (Phys(rd), Phys(rs)) => self.code.mov_reg_reg(Reg32(rd), Reg32(rs)),
            (Phys(rd), Spill(is)) => self.code.mov_reg_ptr(
                Reg32(rd),
                BaseDisp8 {
                    base: RBP,
                    disp: spill_stack_disp(is) as i8,
                },
            ),
            (Spill(id), Phys(rs)) => self.code.mov_ptr_reg(
                BaseDisp8 {
                    base: RBP,
                    disp: spill_stack_disp(id) as i8,
                },
                Reg32(rs),
            ),
            (Spill(id), Spill(is)) => self
                .code
                .mov_reg_ptr(
                    Reg32(RAX),
                    BaseDisp8 {
                        base: RBP,
                        disp: spill_stack_disp(is) as i8,
                    },
                )
                .mov_ptr_reg(
                    BaseDisp8 {
                        base: RBP,
                        disp: spill_stack_disp(id) as i8,
                    },
                    Reg32(RAX),
                ),
            _ => panic!(),
        };
        self
    }

    fn mov_imm(&mut self, dest: VReg, imm: i16) -> &mut Self {
        match self.reg_alloc.get(dest) {
            Phys(rd) => self.code.mov_reg64_imm32(rd, imm as i32),
            Spill(ri) => {
                self.code
                    .mov_ptr64_imm32_disp8(RBP, imm as i32, spill_stack_disp(ri) as i8)
            }
            Unmapped => panic!(),
        };
        self
    }

    /// Load value to register from absolute address
    fn ldr_abs(&mut self, dest: VReg, addr: u32) -> &mut Self {
        match self.reg_alloc.get(dest) {
            // TODO are unsigned to signed offset conversions going to be a problem?
            Phys(r) => {
                self.code.mov_reg_ptr(
                    Reg32(r),
                    BaseDisp32 {
                        base: RDX,
                        disp: addr as i32,
                    },
                );
            }
            Spill(i) => {
                // TODO - assuming here we can't spill past size of i8, i.e. 127 bytes. Should have
                // a check for that at some point (or stop using i8s)
                self.code
                    .mov_reg_ptr(
                        Reg32(RAX),
                        BaseDisp32 {
                            base: RDX,
                            disp: addr as i32,
                        },
                    )
                    .mov_ptr_reg(
                        BaseDisp32 {
                            base: RBP,
                            disp: spill_stack_disp(i) as i32,
                        },
                        Reg32(RAX),
                    );
            }
            Unmapped => panic!(),
        };
        self
    }

    /// Load value to register from address in pointer register plus immediate offset
    fn ldr_rel_imm(&mut self, dest: VReg, ptr: VReg, offset: i32) -> &mut Self {
        match (self.reg_alloc.get(dest), self.reg_alloc.get(ptr)) {
            (Phys(rd), Phys(rs)) => {
                self.code.mov_reg_ptr(
                    Reg32(rd),
                    SIBDisp32 {
                        base: RDX,
                        scale: 1,
                        index: rs,
                        disp: offset,
                    },
                );
            }
            (Phys(rd), Spill(is)) => {
                self.code
                    .mov_reg_ptr(
                        Reg32(RAX),
                        BaseDisp8 {
                            base: RBP,
                            disp: spill_stack_disp(is) as i8,
                        },
                    )
                    .mov_reg_ptr(
                        Reg32(rd),
                        SIBDisp32 {
                            base: RDX,
                            scale: 1,
                            index: RAX,
                            disp: offset,
                        },
                    );
            }
            (Spill(id), Phys(rs)) => {
                self.code
                    .mov_reg_ptr(
                        Reg32(RAX),
                        SIBDisp32 {
                            base: RDX,
                            scale: 1,
                            index: rs,
                            disp: offset,
                        },
                    )
                    .mov_ptr_reg(
                        BaseDisp8 {
                            base: RBP,
                            disp: spill_stack_disp(id) as i8,
                        },
                        Reg32(RAX),
                    );
            }
            (Spill(id), Spill(is)) => {
                self.code
                    .mov_reg_ptr(
                        Reg32(RAX),
                        BaseDisp8 {
                            base: RBP,
                            disp: spill_stack_disp(is) as i8,
                        },
                    )
                    .mov_reg_ptr(
                        Reg32(RAX),
                        SIBDisp32 {
                            base: RDX,
                            scale: 1,
                            index: RAX,
                            disp: offset,
                        },
                    )
                    .mov_ptr_reg(
                        BaseDisp8 {
                            base: RBP,
                            disp: spill_stack_disp(id) as i8,
                        },
                        Reg32(RAX),
                    );
            }
            _ => panic!(),
        };
        self
    }

    /// Load value to register from address in pointer register plus index register
    fn ldr_rel_ind_imm(&mut self, dest: VReg, ptr: VReg, ind: VReg, offset: i32) -> &mut Self {
        match self.reg_alloc.get(ptr) {
            Phys(r) => self.code.mov_reg_reg(Reg32(RAX), Reg32(r)),
            Spill(i) => self.code.mov_reg_ptr(
                Reg32(RAX),
                BaseDisp8 {
                    base: RBP,
                    disp: spill_stack_disp(i) as i8,
                },
            ),
            Unmapped => panic!(),
        };
        match self.reg_alloc.get(ind) {
            Phys(r) => self.code.add_reg_reg(Reg32(RAX), Reg32(r)),
            Spill(i) => self.code.add_reg_ptr(
                Reg32(RAX),
                BaseDisp8 {
                    base: RBP,
                    disp: spill_stack_disp(i) as i8,
                },
            ),
            Unmapped => panic!(),
        };
        match self.reg_alloc.get(dest) {
            Phys(r) => {
                if offset == 0 {
                    self.code.mov_reg_ptr(
                        Reg32(r),
                        SIBNoDisp {
                            base: RDX,
                            scale: 1,
                            index: RAX,
                        },
                    );
                } else {
                    // Does this exist?
                    self.code.mov_reg_ptr(
                        Reg32(r),
                        SIBDisp32 {
                            base: RDX,
                            scale: 1,
                            index: RAX,
                            disp: offset,
                        },
                    );
                }
            }
            Spill(i) => {
                if offset == 0 {
                    self.code.mov_reg_ptr(
                        Reg32(RAX),
                        SIBNoDisp {
                            base: RDX,
                            scale: 1,
                            index: RAX,
                        },
                    );
                } else {
                    self.code.mov_reg_ptr(
                        Reg32(RAX),
                        SIBDisp32 {
                            base: RDX,
                            scale: 1,
                            index: RAX,
                            disp: offset,
                        },
                    );
                }
                self.code.mov_ptr_reg(
                    BaseDisp8 {
                        base: RBP,
                        disp: spill_stack_disp(i) as i8,
                    },
                    Reg32(RAX),
                );
            }
            Unmapped => panic!(),
        };
        self
    }

    fn ret(&mut self) -> &mut Self {
        self.code.ret();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{AssemblerX64, RegAllocation};
    use crate::cpu::ARM7;
    use crate::ir::VReg::*;

    #[test]
    fn test_mov() {
        let mut cpu = ARM7::new();
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.gen_prologue()
            .mov_imm(R0, 4958) // phys
            .mov_reg(R6, R0) // phys -> phys
            .mov_imm(SP, 193) // spill
            .mov_reg(LR, SP) // spill -> spill
            .mov_reg(PC, R6) // phys -> spill
            .mov_reg(R3, PC) // spill -> phys
            .gen_epilogue();
        let f = asm.get_exec_buffer();
        dbg!(cpu.vregs);
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        dbg!(cpu.vregs);
        assert_eq!(cpu.vregs[R0 as usize], 4958);
        assert_eq!(cpu.vregs[R6 as usize], 4958);
        assert_eq!(cpu.vregs[SP as usize], 193);
        assert_eq!(cpu.vregs[LR as usize], 193);
        assert_eq!(cpu.vregs[PC as usize], 4958);
        assert_eq!(cpu.vregs[R3 as usize], 4958);
    }

    fn setup_cpu_test_data(cpu: &mut ARM7) {
        cpu.vregs[SP as usize] = 80;
        cpu.vregs[R3 as usize] = 81;
        cpu.vregs[R4 as usize] = 16;
        cpu.vregs[R11 as usize] = 4;
        cpu.mem[80] = 0xa3;
        cpu.mem[81] = 0x03;
        cpu.mem[82] = 0xf1;
        cpu.mem[83] = 0x4e;
        cpu.mem[84] = 0xbb;
        cpu.mem[85] = 0x73;
        cpu.mem[86] = 0xda;
        cpu.mem[87] = 0x09;
        cpu.mem[96] = 0x6c;
        cpu.mem[97] = 0x78;
        cpu.mem[98] = 0xff;
        cpu.mem[99] = 0x32;
        cpu.mem[10100] = 0x1a;
        cpu.mem[10101] = 0x6b;
        cpu.mem[10102] = 0x80;
        cpu.mem[10103] = 0xcc;
    }

    #[test]
    fn test_ldr_abs() {
        let mut cpu = ARM7::new();
        setup_cpu_test_data(&mut cpu);
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.gen_prologue()
            .ldr_abs(R0, 80) // phys
            .ldr_abs(PC, 98) // spill
            .gen_epilogue();
        asm.hex_dump();
        dbg!(cpu.vregs);
        let f = asm.get_exec_buffer();
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        dbg!(cpu.vregs);
        assert_eq!(cpu.vregs[R0 as usize], 0x4ef103a3);
        assert_eq!(cpu.vregs[PC as usize], 0x000032ff);
    }

    #[test]
    fn test_ldr_rel_ind_imm() {
        let mut cpu = ARM7::new();
        setup_cpu_test_data(&mut cpu);
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.gen_prologue()
            .ldr_rel_ind_imm(R1, R3, R4, 0) // ppp
            .ldr_rel_ind_imm(R2, R3, R11, 0) // pps
            .ldr_rel_ind_imm(R5, SP, R4, 0) // psp
            .ldr_rel_ind_imm(R6, SP, R11, 0) // pss
            .ldr_rel_ind_imm(LR, SP, R4, 0) // ssp
            .ldr_rel_ind_imm(R7, SP, R11, 10016)
            .ldr_rel_ind_imm(R8, SP, R4, -10)
            .gen_epilogue();
        asm.hex_dump();
        dbg!(cpu.vregs);
        let f = asm.get_exec_buffer();
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        assert_eq!(cpu.vregs[R1 as usize], 0x0032ff78);
        assert_eq!(cpu.vregs[R2 as usize], 0x0009da73);
        assert_eq!(cpu.vregs[R5 as usize], 0x32ff786c);
        assert_eq!(cpu.vregs[R6 as usize], 0x09da73bb);
        assert_eq!(cpu.vregs[LR as usize], 0x32ff786c);
        assert_eq!(cpu.vregs[R7 as usize], 0xcc806b1a);
        assert_eq!(cpu.vregs[R8 as usize], 0x000009da);
    }

    #[test]
    fn test_ldr_rel_imm() {
        let mut cpu = ARM7::new();
        setup_cpu_test_data(&mut cpu);
        let mut asm = AssemblerX64::new(RegAllocation::default());
        asm.gen_prologue()
            .ldr_rel_imm(R9, R3, -1) // pp
            .ldr_rel_imm(R10, SP, 16) // ps
            .ldr_rel_imm(PC, R4, 64) // sp
            .ldr_rel_imm(R12, SP, 2) // ss
            .gen_epilogue();
        asm.hex_dump();
        dbg!(cpu.vregs);
        let f = asm.get_exec_buffer();
        f.call(cpu.vreg_base_ptr(), cpu.mem_base_ptr());
        assert_eq!(cpu.vregs[R9 as usize], 0x4ef103a3);
        assert_eq!(cpu.vregs[R10 as usize], 0x32ff786c);
        assert_eq!(cpu.vregs[PC as usize], 0x4ef103a3);
        assert_eq!(cpu.vregs[R12 as usize], 0x73bb4ef1);
    }

    fn test_call_reg() {}
}
