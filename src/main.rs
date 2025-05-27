use std::fs::File;
use std::io::Write;

use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};

use cranelift::{
    codegen::{
        ir::{AbiParam, Function, Signature, UserFuncName, types},
        isa::CallConv,
    },
    prelude::InstBuilder,
};

use cranelift::codegen::{Context, settings};
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

// fn run_program(code_buffer: &[u8]) {
//     let mut buffer = memmap2::MmapOptions::new()
//         .len(code_buffer.len())
//         .map_anon()
//         .unwrap();
//     buffer.copy_from_slice(code_buffer);
//     let buffer = buffer.make_exec().unwrap();
//     let x = unsafe {
//         let code_fn: unsafe extern "C" fn(usize) -> usize = std::mem::transmute(buffer.as_ptr());

//         code_fn(1)
//     };
//     println!("out: {}", x);
// }

// fn dump_to_binary() {
//     // read binary with:
//     // `/opt/homebrew/opt/binutils/bin/gobjdump -D -b binary -m aarch64 dump.bin`

//     // let mut ctx = Context::for_function(func);
//     // let code = ctx.compile(&*isa, &mut ControlPlane::default()).unwrap();

//     // std::fs::write("dump.bin", code.code_buffer()).unwrap();
// }

fn test() {
    let mut flags_builder = settings::builder();
    // Enable Position Independent Code (PIC)
    flags_builder.set("is_pic", "true").unwrap();
    let isa_flags = settings::Flags::new(flags_builder);

    let isa_builder = cranelift_native::builder().unwrap();
    let isa = isa_builder.finish(isa_flags).unwrap();

    let pointer_type = isa.pointer_type();

    // Signature for tempo_entry: takes a pointer, returns I64
    let mut sig_tempo_entry = Signature::new(CallConv::SystemV);
    sig_tempo_entry.params.push(AbiParam::new(pointer_type)); // Still takes the string pointer, though unused
    sig_tempo_entry.returns.push(AbiParam::new(types::I64));   // Returns an integer

    let mut func = Function::with_name_signature(UserFuncName::default(), sig_tempo_entry.clone());

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let block = builder.create_block();
    builder.seal_block(block);

    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);

    // Signature for the C function get_two_from_c: takes nothing, returns I64
    let mut sig_get_two = Signature::new(CallConv::SystemV);
    sig_get_two.returns.push(AbiParam::new(types::I64));

    let mut obj_module = ObjectModule::new(
        ObjectBuilder::new(
            isa.clone(),
            "main_module_name", // Arbitrary internal name for the object module
            cranelift_module::default_libcall_names(),
        )
        .unwrap(),
    );

    // Declare get_two_from_c as an imported function
    let callee_get_two_id = obj_module
        .declare_function("get_two_from_c", Linkage::Import, &sig_get_two)
        .unwrap();
    let local_callee_get_two = obj_module.declare_func_in_func(callee_get_two_id, builder.func);

    // Call get_two_from_c()
    let call_inst = builder.ins().call(local_callee_get_two, &[]);
    let call_result = builder.inst_results(call_inst)[0];

    // Return the result of the call
    builder.ins().return_(&[call_result]);
    builder.finalize();

    // Declare tempo_entry as an exported function
    let func_id_tempo_entry = obj_module
        .declare_function("tempo_entry", Linkage::Export, &sig_tempo_entry)
        .unwrap();
    let mut ctx = Context::for_function(func);
    obj_module.define_function(func_id_tempo_entry, &mut ctx).unwrap();

    let product = obj_module.finish();
    let obj_bytes = product.emit().unwrap();

    File::create("tempo.o")
        .unwrap()
        .write_all(&obj_bytes)
        .unwrap();
}

fn main() {
    test();
}
