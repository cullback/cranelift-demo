use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

pub fn parse_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap();
    let pairs = match MyParser::parse(Rule::program, &content) {
        Ok(pairs) => pairs,
        Err(e) => panic!("Parsing error: {}", e),
    };

    println!("{:?}", pairs);
}
