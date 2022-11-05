// The JIT compiler interface used to implement the CodeGen trait, abstracts over the x86_64
// interface handled by the Emitter
use crate::asm::execbuffer::ExecBuffer;
use crate::ir::VReg;

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
pub struct Value {
    dtype: DataType,
    id: usize,
}

/// Represents a jump target
pub struct Label {
    index: usize
}

/// Used for memory reads/writes, but not sure how these will be created/used just yet
pub struct Address {}

/// Planning to replace the Assembler interface with this. Will compile single code blocks only,
/// i.e. no control flow, functions, etc.
pub struct CompilerX64 {
        
}

impl CompilerX64 {
    pub fn compile() -> ExecBuffer {
        todo!();
    }

    /// Create a new 32-bit named valued. User provided names are used for the context switch
    /// between compiled code and emulated state
    pub fn named_word(name: VReg) -> Value {
        todo!();
    }

    /// Create a new 16-bit named valued. User provided names are used for the context switch
    /// between compiled code and emulated state
    pub fn named_halfword(name: VReg) -> Value {
        todo!();
    }

    /// Create a new 16-bit named valued. User provided names are used for the context switch
    /// between compiled code and emulated state
    pub fn named_byte(name: VReg) -> Value {
        todo!()
    }

    /// Create new temporary 32-bit value. Temporary values are used for operations but won't
    /// persist past the execution of the code block
    pub fn temp_word() -> Value {
        todo!()
    }

    /// Create new temporary 16-bit value. Temporary values are used for operations but won't
    /// persist past the execution of the code block
    pub fn temp_halfword() -> Value {
        todo!()
    }

    /// Create new temporary 8-bit value. Temporary values are used for operations but won't
    /// persist past the execution of the code block   
    pub fn temp_byte() -> Value {
        todo!()
    }

    pub fn gen_label() -> Label {
        todo!()
    }

    pub fn bind_label() -> Label {
        todo!()
    }

    /// Does it make sense to have different funcs for each of these?
    pub fn add_word(dest: Value, op: Value) {
        todo!();
    }

    pub fn add_halfword(dest: Value, op: Value) {
        todo!();
    }

    pub fn add_byte(dest: Value, op: Value) {
        todo!();
    }

    pub fn read_mem(dest: Value, addr: Address) {
        todo!();
    }

    pub fn write_mem(addr: Address, src: Value) {
        todo!();
    }
}
