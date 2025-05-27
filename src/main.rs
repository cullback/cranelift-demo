use std::fs::File;
use std::io::Write;

use parser::parse_file;
use ast::compile_program_to_object_bytes; // Import the new function

mod ast;
mod parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];

    // This assumes parse_file now returns Result<Program<'_>, String>
    // and that this change was part of commit dc55004.
    // If not, parse_file needs to be updated.
    // For now, I'll proceed as if parse_file is already returning a Result.
    // The parser.rs provided in the chat does not return a Result yet.
    // This will be a point of failure if parser.rs isn't updated.
    //
    // Let's assume for now that parse_file is updated like this in a previous step:
    // pub fn parse_file(path: &str) -> Result<crate::ast::Program<'_>, String> { ... }

    match parse_file(path) {
        Ok(ast_root) => {
            // println!("{:#?}", ast_root); // Optionally print AST
            match compile_program_to_object_bytes(&ast_root) {
                Ok(obj_bytes) => {
                    let output_path = "tempo.o";
                    match File::create(output_path) {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(&obj_bytes) {
                                eprintln!("Error writing to {}: {}", output_path, e);
                                std::process::exit(1);
                            }
                            println!("Successfully compiled to {}", output_path);
                        }
                        Err(e) => {
                            eprintln!("Error creating file {}: {}", output_path, e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Compilation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }
    }
}
