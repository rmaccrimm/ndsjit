use std::{error::Error, mem, ptr};

use cranelift::prelude::EntityRef;
use cranelift::prelude::MemFlags;
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::ArgumentPurpose;
use cranelift_codegen::ir::{AbiParam, GlobalValueData, InstBuilder, Value};
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::Variable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{self, Module};

use ndsjit::asm::block_translator::BlockTranslator;

#[repr(C, packed)]
#[derive(Default, Copy, Clone, Debug)]
struct Regs {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut translator = BlockTranslator::new();
    let code = vec![];
    translator.translate(&code);
    return Ok(());

    // OLD MAIN ------------------------------------------
    let mut vm_state = Regs::default();
    vm_state.r0 = 113;
    vm_state.r1 = 999;
    dbg!(vm_state);

    let jit_builder =
        JITBuilder::new(cranelift_module::default_libcall_names()).map_err(|e| e.to_string())?;
    let mut module = JITModule::new(jit_builder);

    let mut ctx = module.make_context();

    let ptr_type = module.target_config().pointer_type();
    dbg!(ptr_type);
    ctx.func
        .signature
        .params
        .push(AbiParam::special(ptr_type, ArgumentPurpose::VMContext));
    ctx.func.signature.returns.push(AbiParam::new(I32));

    let mut builder_ctx = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let vmctx = builder.create_global_value(GlobalValueData::VMContext);
        let r0 = Variable::new(0);
        builder.declare_var(r0, I32);

        // let r0 = builder.create_global_value(GlobalValueData::Load {
        //     base: vmctx,
        //     offset: Offset32::new(0),
        //     global_type: I32,
        //     readonly: false,
        // });

        let block0 = builder.create_block();
        builder.append_block_params_for_function_params(block0);

        builder.switch_to_block(block0);
        builder.seal_block(block0);
        let base = builder.ins().global_value(ptr_type, vmctx);
        {
            let tmp = builder.ins().load(I32, MemFlags::new(), base, 0);
            builder.def_var(r0, tmp);
        }
        {
            let arg = builder.use_var(r0);
            let const_ = builder.ins().iconst(I32, 10);
            let tmp = builder.ins().iadd(arg, const_);
            builder.def_var(r0, tmp);
        }
        {
            let arg = builder.use_var(r0);
            builder.ins().store(MemFlags::new(), arg, base, 0);
            // let tmp = builder.ins().uload32(MemFlags::new(), base, 0);
            // builder.def_var(r0, tmp);
            // let ret = builder.use_var(r0);
            builder.ins().return_(&[arg]);
        }
        builder.finalize();
    }

    // let flags = settings::Flags::new(settings::builder());
    // let res = verify_function(&func, &flags);
    // println!("{}", func.display());

    let flags = settings::Flags::new(settings::builder());
    verify_function(&ctx.func, &flags)?;
    println!("{}", ctx.func.display());

    // let mut sig = Signature::new(CallConv::SystemV);
    // sig.params
    //     .push(AbiParam::special(ptr_type, ArgumentPurpose::VMContext));
    // sig.returns.push(AbiParam::new(I32));

    let func_id = module.declare_anonymous_function(&ctx.func.signature)?;
    // .map_err(|e| e.to_string())?;
    module.define_function(func_id, &mut ctx)?;
    // .map_err(|e| e.to_string())?;

    module.clear_context(&mut ctx);
    module.finalize_definitions().unwrap();

    let func_ptr = module.get_finalized_function(func_id);

    unsafe {
        let func: unsafe extern "C" fn(*mut Regs) -> i32 = mem::transmute(func_ptr);
        dbg!(func(ptr::addr_of_mut!(vm_state)));
    }
    dbg!(vm_state);

    Ok(())
}
