// Remove unused imports related to Cranelift
// use std::fs::File;
// use std::io::Write;
// use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
// use cranelift::prelude::Configurable; // No longer needed
// use cranelift::{
//     codegen::{
//         ir::{AbiParam, Function, Signature, UserFuncName, types},
//         isa::CallConv,
//     },
//     prelude::InstBuilder,
// };
// use cranelift::codegen::{Context, settings};
// use cranelift_module::{Linkage, Module};
// use cranelift_object::{ObjectBuilder, ObjectModule};

use parser::parse_file; // Keep this

mod ast;
mod parser;

// The test() function and other Cranelift related functions are removed.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];

    match parse_file(path) {
        Ok(ast_root) => {
            // Pretty print the AST
            println!("{:#?}", ast_root);
        }
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }
    }
}
