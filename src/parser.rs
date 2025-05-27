use std::{fs::File, io::Write};

use pest::Parser;
use pest_derive::Parser;

use crate::ast::{compile_program_to_object_bytes, parse_program};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

pub fn parse_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap();
    let pairs = match MyParser::parse(Rule::program, &content) {
        Ok(pairs) => pairs,
        Err(e) => {
            eprintln!("Parsing error: {}", e);
            std::process::exit(1);
        }
    };

    let ast_root = match parse_program(pairs) {
        Ok(ast_root) => ast_root,
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }
    };
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
