use pest::Parser;
use pest_derive::Parser;

use crate::ast::{parse_program, Program}; // Import Program
use crate::parser::Rule; // Import Rule if not already globally visible, or ensure MyParser::parse uses self::Rule

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

pub fn parse_file(path: &str) -> Result<Program<'_>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
    let pairs = match MyParser::parse(Rule::program, &content) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Parsing error: {}", e)),
    };

    // parse_program expects the inner pairs of the 'program' rule.
    // MyParser::parse(Rule::program, ...) already returns these inner pairs.
    parse_program(pairs)
}
