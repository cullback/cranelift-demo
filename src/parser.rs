use pest::Parser;
use pest_derive::Parser;

use crate::ast::parse_program;

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

    match parse_program(pairs) {
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
