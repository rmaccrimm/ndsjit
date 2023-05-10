use std::f32::consts::E;

use cranelift::prelude::{AbiParam, EntityRef, GlobalValueData, InstBuilder, MemFlags};
use cranelift_codegen::ir::{
    immediates::Offset32, types::I32, ArgumentPurpose, Function, GlobalValue,
};
use cranelift_codegen::{settings, verify_function};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;

use super::instruction_translator::translate_instruction;
use crate::disasm::armv4t::{Instruction, Register};

pub struct BlockTranslator {
    builder_ctx: FunctionBuilderContext,
    register_vars: Vec<Variable>,
}

impl BlockTranslator {
    pub fn new() -> Self {
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            register_vars: vec![],
        }
    }

    pub fn translate(&mut self, code: &Vec<Instruction>) {
        let jit_builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(jit_builder);
        let mut ctx = module.make_context();
        let ptr_type = module.target_config().pointer_type();

        ctx.func
            .signature
            .params
            .push(AbiParam::special(ptr_type, ArgumentPurpose::VMContext));
        ctx.func.signature.returns.push(AbiParam::new(I32));

        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_ctx);

        {
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let vmctx = builder.create_global_value(GlobalValueData::VMContext);
            // TODO some kind of trait that governs access to CPU state?
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
            for (i, reg) in registers.iter().enumerate() {
                let var = Variable::new(i);
                builder.declare_var(var, I32);
                self.register_vars.push(var);

                let base = builder.ins().global_value(ptr_type, vmctx);
                let tmp = builder.ins().load(I32, MemFlags::new(), base, 0);
                builder.def_var(var, tmp);
            }

            let r2 = builder.use_var(self.register_vars[2]);
            let const_ = builder.ins().iconst(I32, 10);
            let tmp = builder.ins().iadd(r2, const_);
            builder.def_var(self.register_vars[2], tmp);

            for (i, &var) in self.register_vars.iter().enumerate() {
                let base = builder.ins().global_value(ptr_type, vmctx);
                let arg = builder.use_var(var);
                builder
                    .ins()
                    .store(MemFlags::new(), arg, base, 4 * (i as i32));
            }
            let const_ = builder.ins().iconst(I32, 0);
            builder.ins().return_(&[const_]);
            builder.finalize();
        }

        let flags = settings::Flags::new(settings::builder());
        verify_function(&ctx.func, &flags).unwrap();
        println!("{}", ctx.func.display());
    }
}
