pub mod emitter;
pub mod execbuffer;

use std::collections::HashMap;

// The JIT compiler interface used to implement the CodeGen trait, abstracts over the x86_64
// interface handled by the Emitter
use execbuffer::ExecBuffer;

pub use emitter::Label;

use self::emitter::EmitterX64;

/// Should address be one of these?
pub enum DataType {
    /// 32 bits
    Word,

    /// 16 bit
    Halfword,

    /// 8 bits
    Byte,
}

/// Represents a register or temp value. Under the hood will either be mapped to a physical register
/// or allocated on the stack
pub struct Variable {
    dtype: DataType,
    id: usize,
}

pub enum Value {
    Variable(Variable),
    ImmediateData(i32),
}

/// Planning to replace the Assembler interface with this. Will compile single code blocks only,
/// i.e. no control flow, functions, etc.
pub struct CompilerX64 {
    e: EmitterX64,
    vars: HashMap<usize, Variable>,
}

impl CompilerX64 {
    pub fn new() -> CompilerX64 {
        CompilerX64 {
            e: EmitterX64::new(),
            vars: HashMap::new(),
        }
    }

    pub fn compile() -> ExecBuffer {
        todo!();
    }

    pub fn evaluate_flags() {
        todo!();
    }

    /// Create a new 32-bit named valued. User provided names are used for the context switch
    /// between compiled code and emulated state
    pub fn var_word(&mut self, id: usize) -> Variable {
        todo!();
    }

    pub fn label(&mut self) -> Label {
        self.e.gen_label()
    }

    pub fn bind(&mut self, label: Label) -> &mut Self {
        self.e.bind_label(label);
	self
    }

    /// Does it make sense to have different funcs for each of these?
    pub fn add_reg(&mut self, dest: Variable, rn: Variable, update_flags: bool) -> &mut Self {
        todo!();
    }

    pub fn add_imm(&mut self, dest: Variable, imm: i32, update_flags: bool) -> &mut Self {
	todo!();
    }
    
    pub fn jz(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jnz(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jc(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jnc(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jn(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jnn(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jv(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jnv(&mut self, label: Label) -> &mut Self {
        todo!();
    }

    pub fn jmp(&mut self, label: Label) -> &mut Self {
        todo!();
    }
}
