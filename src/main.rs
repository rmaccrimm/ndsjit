use cranelift::prelude::InstBuilder;
use ndsjit::cpu::arm7::InstrMode;
use std::fmt::Binary;
use std::mem;

use cranelift::codegen::ir::immediates::Offset32;
use cranelift::codegen::ir::types::{I32, I64};
use cranelift::codegen::ir::{AbiParam, Endianness, Function, MemFlags, Signature, UserFuncName};
use cranelift::codegen::isa::CallConv;
use cranelift::codegen::verifier::{verify_function, VerifierErrors};
use cranelift::codegen::{dbg, settings, Context};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{self, Module};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct VMState {
    pub regs: [u32; 4],
}

struct CompiledCode {
    ptr: *const u8,
}

impl CompiledCode {
    fn from_ptr(ptr: *const u8) -> Self {
        Self { ptr }
    }

    unsafe fn call(&self, vm_state: &mut VMState) {
        let func: unsafe extern "C" fn(*mut u32) = mem::transmute(self.ptr);
        func(vm_state.regs.as_mut_ptr());
    }
}

pub struct Instruction {}

struct InstrTranslator {
    pub module: JITModule,
    pub ctx: Context,
    pub builder_ctx: FunctionBuilderContext,
}

impl Default for InstrTranslator {
    fn default() -> Self {
        let jit_builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(jit_builder);
        let ctx = module.make_context();

        Self {
            module,
            ctx,
            builder_ctx: FunctionBuilderContext::new(),
        }
    }
}

impl InstrTranslator {
    fn translate_segment(&mut self, decoder: &mut BinaryDecoder) {
        let int_type = self.module.target_config().pointer_type();
        self.ctx.func.signature.params.push(AbiParam::new(int_type));
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);
    }

    fn translate_instr(instr: Instruction, builder: &mut FunctionBuilder) {
        todo!()
    }
}

struct BinaryDecoder<'a> {
    data: &'a [u8],
    pos: usize,
    mode: InstrMode,
}

impl BinaryDecoder<'_> {
    fn next_instr(&mut self) {
        todo!();
    }
}

fn main() -> Result<(), String> {
    // Goal - state transfer into/out of Function
    let mut vm_state = VMState { regs: [0; 4] };
    dbg!(vm_state);

    let jit_builder =
        JITBuilder::new(cranelift_module::default_libcall_names()).map_err(|e| e.to_string())?;
    let mut module = JITModule::new(jit_builder);

    let mut sig = Signature::new(CallConv::Fast);
    // sig.params.push(AbiParam::new(I64));

    let mut ctx = module.make_context();

    let int_type = module.target_config().pointer_type();
    ctx.func.signature.params.push(AbiParam::new(int_type));

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);

    builder.switch_to_block(block);
    builder.seal_block(block);

    let base = builder.block_params(block)[0];

    let offset = Offset32::new(0);
    let flags = MemFlags::trusted().with_endianness(Endianness::Little);
    let reg0 = builder.ins().load(I32, flags, base, offset);

    let imm = builder.ins().iconst(I32, 17);
    let added = builder.ins().iadd(reg0, imm);
    builder.ins().store(flags, added, base, offset);
    builder.ins().return_(&[]);

    builder.finalize();

    let flags = settings::Flags::new(settings::builder());
    verify_function(&ctx.func, &flags).map_err(|e| e.to_string())?;
    println!("{}", ctx.func.display());

    let func_id = module
        .declare_anonymous_function(&mut sig)
        .map_err(|e| e.to_string())?;
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| e.to_string())?;

    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();

    let func_ptr = module.get_finalized_function(func_id);

    unsafe {
        let func: unsafe extern "C" fn(*mut u32) = mem::transmute(func_ptr);
        func(vm_state.regs.as_mut_ptr());
    }
    dbg!(vm_state);

    Ok(())
}
