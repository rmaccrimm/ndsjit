use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{ChildStdin, Command, Stdio},
};

use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{
    AbiParam, Function, GlobalValue, GlobalValueData, InstBuilder, Signature, UserFuncName,
};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_codegen::{entity::EntityRef, ir::ArgumentPurpose};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

#[repr(C, packed)]
struct Regs {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1st step - state transfer: Need to build a block(?) with all of the registers (used?) and
    // convert them to cranelift vars.
    // For now - probably just assume all will be used and let optimizer get rid of them

    let mut sig = Signature::new(CallConv::SystemV);
    sig.params
        .push(AbiParam::special(I64, ArgumentPurpose::VMContext));
    sig.returns.push(AbiParam::new(I64));
    let mut fn_builder_ctx = FunctionBuilderContext::new();
    let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
    {
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);
        let vmctx = builder.create_global_value(GlobalValueData::VMContext);

        let block0 = builder.create_block();
        builder.append_block_params_for_function_returns(block0);

        builder.switch_to_block(block0);
        builder.seal_block(block0);
        {
            let r0 = builder.ins().global_value(I64, vmctx);
            builder.ins().return_(&[r0]);
        }
        builder.finalize();
    }

    let flags = settings::Flags::new(settings::builder());
    let res = verify_function(&func, &flags);
    println!("{}", func.display());
    if let Err(errors) = res {
        panic!("{}", errors);
    };
    Ok(())
}
