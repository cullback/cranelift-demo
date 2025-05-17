use cranelift::codegen::control::ControlPlane;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};

use cranelift::{
    codegen::{
        ir::{AbiParam, Function, Signature, UserFuncName, types},
        isa::CallConv,
    },
    prelude::InstBuilder,
};

use cranelift::codegen::{Context, settings};

fn run_program(code_buffer: &[u8]) {
    let mut buffer = memmap2::MmapOptions::new()
        .len(code_buffer.len())
        .map_anon()
        .unwrap();
    buffer.copy_from_slice(code_buffer);
    let buffer = buffer.make_exec().unwrap();
    let x = unsafe {
        let code_fn: unsafe extern "C" fn(usize) -> usize = std::mem::transmute(buffer.as_ptr());

        code_fn(1)
    };
    println!("out: {}", x);
}

fn test() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(types::I64));
    sig.returns.push(AbiParam::new(types::I64));

    let mut func = Function::with_name_signature(UserFuncName::default(), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let block = builder.create_block();
    builder.seal_block(block);

    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);

    let arg = builder.block_params(block)[0];
    let v1 = builder.ins().iconst(types::I64, 5);
    let v2 = builder.ins().iadd(arg, v1);
    builder.ins().return_(&[v2]);

    builder.finalize();

    println!("{}", func.display());

    let builder = settings::builder();
    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {}", msg);
    });
    let isa = isa_builder.finish(settings::Flags::new(builder)).unwrap();

    let mut ctx = Context::for_function(func);
    let code = ctx.compile(&*isa, &mut ControlPlane::default()).unwrap();

    std::fs::write("dump.bin", code.code_buffer()).unwrap();
}

fn main() {
    println!("Hello, world!");
    test();
}
