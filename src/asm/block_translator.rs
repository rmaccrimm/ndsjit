use cranelift::prelude::{AbiParam, EntityRef, GlobalValueData, InstBuilder, MemFlags};
use cranelift_codegen::ir::{
    types::{I32, I64},
    ArgumentPurpose, GlobalValue,
};
use cranelift_codegen::{settings, verify_function};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use strum::IntoEnumIterator;

use super::TranslationError;
use std::mem;

use crate::{
    asm::instruction_translator::translate_instruction,
    disasm::armv4t::{Instruction, Register},
};

use super::instruction_translator::{get_reg_index, TranslationState};

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

    /// TODO - more specific error type?
    pub fn translate(&mut self, code: &Vec<Instruction>) -> Result<*const u8, TranslationError> {
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

        // Start loop
        for instr in code.iter() {
            translate_instruction(instr, &self.state, &mut builder)?;
        }

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

fn get_reg_offset(reg: Register) -> i32 {
    // TODO -  generic reg size
    (get_reg_index(reg) * mem::size_of::<u32>()) as i32
}

fn gen_prologue(vmctx: GlobalValue, state: &mut TranslationState, builder: &mut FunctionBuilder) {
    // TODO some kind of trait that governs access to CPU state
    // Create a re-usable variable for each of the CPU registers
    // TODO - some sort of context/environment managing this ptr type and other things like it
    let base = builder.ins().global_value(I64, vmctx);
    for (i, reg) in Register::iter().enumerate() {
        let var = Variable::new(i);
        builder.declare_var(var, I32);
        state.register_vars.push(var);
        let tmp = builder
            .ins()
            .load(I32, MemFlags::new(), base, get_reg_offset(reg));
        builder.def_var(var, tmp);
    }
}

fn gen_epilogue(vmctx: GlobalValue, state: &TranslationState, builder: &mut FunctionBuilder) {
    let base = builder.ins().global_value(I64, vmctx);
    for (i, &var) in state.register_vars.iter().enumerate() {
        let arg = builder.use_var(var);
        builder.ins().store(
            MemFlags::new(),
            arg,
            base,
            get_reg_offset(Register::try_from(i as u32).unwrap()),
        );
    }
    let const_ = builder.ins().iconst(I32, 0);
    builder.ins().return_(&[const_]);
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::BlockTranslator;
    use crate::disasm::armv4t::*;
    use std::{mem, ptr};

    type Func = unsafe extern "C" fn(*mut [u32; 17]) -> i32;
    const V: u32 = 1 << 28;
    const C: u32 = 1 << 29;
    const Z: u32 = 1 << 30;
    const N: u32 = 1 << 31;

    fn add_with_cond(cond: Cond, arg: &mut [u32; 17]) {
        let code = vec![Instruction {
            cond,
            op: Op::ADD,
            operands: vec![
                Operand::Reg(Register::R2),
                Operand::Reg(Register::R2),
                Operand::Imm(99),
            ],
            extra: None,
            set_flags: false,
        }];
        let mut translator = BlockTranslator::new();
        unsafe {
            let func_ptr = translator.translate(&code).unwrap();
            let func: Func = mem::transmute(func_ptr);
            func(ptr::addr_of_mut!(*arg));
        }
    }

    #[test]
    fn test_EQ_NE() {
        let mut regs = [0u32; 17];
        regs[16] = C | V | N;
        add_with_cond(Cond::EQ, &mut regs);
        assert_eq!(regs[2], 0);
        add_with_cond(Cond::NE, &mut regs);
        assert_eq!(regs[2], 99);
        regs[2] = 0;
        regs[16] |= Z;
        add_with_cond(Cond::NE, &mut regs);
        assert_eq!(regs[2], 0);
        add_with_cond(Cond::EQ, &mut regs);
        assert_eq!(regs[2], 99);
    }

    #[test]
    fn test_CS_CC() {}
}
