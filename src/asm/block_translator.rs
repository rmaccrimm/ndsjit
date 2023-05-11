use cranelift::prelude::{AbiParam, EntityRef, GlobalValueData, InstBuilder, IntCC, MemFlags};
use cranelift_codegen::ir::{
    types::{I32, I64},
    ArgumentPurpose, GlobalValue,
};
use cranelift_codegen::{settings, verify_function};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;
use std::error::Error;

use crate::disasm::armv4t::{Instruction, Register};

/// Maybe this will persist between block translations and store the output functions?
struct TranslationState {
    register_vars: Vec<Variable>,
}

/// Plan for code "Blocks" - essentially going to be a list of disassembled instructions and maybe
/// some helper functions for determining things like which registers actually get used
pub struct BlockTranslator {
    builder_ctx: FunctionBuilderContext,
    state: TranslationState,
}

impl BlockTranslator {
    pub fn new() -> Self {
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            state: TranslationState { register_vars: vec![] },
        }
    }

    pub fn translate(&mut self, code: &Vec<Instruction>) -> Result<*const u8, Box<dyn Error>> {
        let jit_builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let mut module = JITModule::new(jit_builder);
        let mut ctx = module.make_context();
        let ptr_type = module.target_config().pointer_type();

        ctx.func
            .signature
            .params
            .push(AbiParam::special(ptr_type, ArgumentPurpose::VMContext));
        ctx.func.signature.returns.push(AbiParam::new(I32));

        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        let vmctx = builder.create_global_value(GlobalValueData::VMContext);
        gen_prologue(vmctx, &mut self.state, &mut builder);

        // let mut current_block = entry_block;

        // Start loop

        // Code for conditional execution
        let instr_block = builder.create_block();
        let next_block = builder.create_block();

        // Still filling prev (entry) block at this point
        let flags = builder.use_var(self.state.register_vars[16]);
        let zero = builder.ins().iconst(I32, 0);
        let res = builder.ins().icmp_imm(IntCC::Equal, flags, 0);
        builder.ins().brif(res, instr_block, &[], next_block, &[]);

        builder.seal_block(instr_block);

        builder.switch_to_block(instr_block);

        // Actual instruction cond
        let r2 = builder.use_var(self.state.register_vars[2]);
        let const_ = builder.ins().iconst(I32, 99);
        let tmp = builder.ins().iadd(r2, const_);
        builder.def_var(self.state.register_vars[2], tmp);
        builder.ins().jump(next_block, &[]);

        builder.seal_block(next_block);
        builder.switch_to_block(next_block);

        // End loop

        gen_epilogue(vmctx, &self.state, &mut builder);
        builder.seal_all_blocks();
        builder.finalize();

        let flags = settings::Flags::new(settings::builder());
        verify_function(&ctx.func, &flags)?;
        println!("{}", ctx.func.display());

        let func_id = module.declare_anonymous_function(&ctx.func.signature)?;
        module.define_function(func_id, &mut ctx)?;

        module.clear_context(&mut ctx);
        module.finalize_definitions()?;

        Ok(module.get_finalized_function(func_id))
    }
}

fn gen_prologue(vmctx: GlobalValue, state: &mut TranslationState, builder: &mut FunctionBuilder) {
    // TODO some kind of trait that governs access to CPU state
    let registers = [
        Register::R0,
        Register::R1,
        Register::R2,
        Register::R3,
        Register::R4,
        Register::R5,
        Register::R6,
        Register::R7,
        Register::R8,
        Register::R9,
        Register::R10,
        Register::R11,
        Register::R12,
        Register::SP,
        Register::LR,
        Register::PC,
        Register::FLAGS,
    ];

    // Create a re-usable variable for each of the CPU registers
    // TODO - some sort of context/environment managing this ptr type and other things like it
    let base = builder.ins().global_value(I64, vmctx);
    for (i, reg) in registers.iter().enumerate() {
        let var = Variable::new(i);
        builder.declare_var(var, I32);
        state.register_vars.push(var);
        let tmp = builder
            .ins()
            .load(I32, MemFlags::new(), base, 4 * (i as i32));
        builder.def_var(var, tmp);
    }
}

fn gen_epilogue(vmctx: GlobalValue, state: &TranslationState, builder: &mut FunctionBuilder) {
    let base = builder.ins().global_value(I64, vmctx);
    for (i, &var) in state.register_vars.iter().enumerate() {
        let arg = builder.use_var(var);
        builder
            .ins()
            .store(MemFlags::new(), arg, base, 4 * (i as i32));
    }
    let const_ = builder.ins().iconst(I32, 0);
    builder.ins().return_(&[const_]);
}
