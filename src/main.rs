use cranelift::prelude::InstBuilder;
use ndsjit::cpu::ARM7;
use std::io::Error;

use cranelift::codegen::ir::immediates::Offset32;
use cranelift::codegen::ir::types::{I32, I64};
use cranelift::codegen::ir::{AbiParam, Endianness, Function, MemFlags, Signature, UserFuncName};
use cranelift::codegen::isa::CallConv;
use cranelift::codegen::settings;
use cranelift::codegen::verifier::{verify_function, VerifierErrors};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit;

#[repr(C, packed)]
struct VMState {
    pub regs: [u32; 4],
}

fn main() -> Result<(), VerifierErrors> {
    // Goal - state transfer into/out of Function
    let mut vm_state = VMState { regs: [0; 4] };

    let mut sig = Signature::new(CallConv::Fast);
    sig.params.push(AbiParam::new(I64));

    let mut ctx = FunctionBuilderContext::new();
    let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
    let mut builder = FunctionBuilder::new(&mut func, &mut ctx);

    let block = builder.create_block();
    let st_addr = Variable::from_u32(0);
    builder.declare_var(st_addr, I64);
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
    verify_function(&func, &flags)?;

    println!("{}", func.display());

    Ok(())
}
