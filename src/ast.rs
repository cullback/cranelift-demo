use pest::{
    Span,
    iterators::{Pair, Pairs},
};

use crate::parser::Rule;

// AST Node types
#[derive(Debug)]
pub enum AstNode<'a> {
    Program(Program<'a>),
    Assignment(Assignment<'a>),
    Expression(Expression<'a>),
    Identifier(Identifier<'a>),
    Number(Number<'a>),
    FunctionCall(FunctionCall<'a>),
    FunctionDefinition(FunctionDefinition<'a>),
    Block(Block<'a>),
}

#[derive(Debug)]
pub struct Program<'a> {
    pub assignments: Vec<Assignment<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct Assignment<'a> {
    pub identifier: Identifier<'a>,
    pub expression: Box<Expression<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct Identifier<'a> {
    pub name: String,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct Number<'a> {
    pub value: i64,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub enum Expression<'a> {
    Number(Number<'a>),
    Identifier(Identifier<'a>),
    FunctionCall(FunctionCall<'a>),
    FunctionDefinition(FunctionDefinition<'a>),
    Block(Block<'a>),
}

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub function_name: Identifier<'a>,
    pub arguments: Vec<Expression<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct FunctionDefinition<'a> {
    pub parameters: Vec<Identifier<'a>>,
    pub body: Box<Expression<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct Block<'a> {
    pub assignments: Vec<Assignment<'a>>,
    pub expression: Box<Expression<'a>>,
    pub span: Span<'a>,
}

// Helper to extract the next inner pair or return an error
fn next_inner_or_err<'a>(
    pairs: &mut pest::iterators::Pairs<'a, Rule>,
    expected_rule_name: &str,
) -> Result<Pair<'a, Rule>, String> {
    pairs
        .next()
        .ok_or_else(|| format!("Expected {} but found nothing", expected_rule_name))
}

// Parsing functions

pub fn parse_identifier<'a>(pair: Pair<'a, Rule>) -> Result<Identifier<'a>, String> {
    if pair.as_rule() != Rule::identifier {
        return Err(format!(
            "Expected identifier, got {:?} for \"{}\"",
            pair.as_rule(),
            pair.as_str()
        ));
    }
    Ok(Identifier {
        name: pair.as_str().to_string(),
        span: pair.as_span(),
    })
}

fn parse_number<'a>(pair: Pair<'a, Rule>) -> Result<Number<'a>, String> {
    if pair.as_rule() != Rule::number {
        return Err(format!(
            "Expected number, got {:?} for \"{}\"",
            pair.as_rule(),
            pair.as_str()
        ));
    }
    let value = pair
        .as_str()
        .parse::<i64>()
        .map_err(|e| format!("Failed to parse number '{}': {}", pair.as_str(), e))?;
    Ok(Number {
        value,
        span: pair.as_span(),
    })
}

fn parse_function_call<'a>(pair: Pair<'a, Rule>) -> Result<FunctionCall<'a>, String> {
    if pair.as_rule() != Rule::function_call {
        return Err(format!(
            "Expected function_call, got {:?} for \"{}\"",
            pair.as_rule(),
            pair.as_str()
        ));
    }
    let fn_call_span = pair.as_span();
    let mut inner_pairs = pair.into_inner();

    let ident_pair = next_inner_or_err(inner_pairs.by_ref(), "function_call identifier")?;
    let function_name = parse_identifier(ident_pair)?;

    let args_pair = next_inner_or_err(inner_pairs.by_ref(), "function_call arguments")?;
    if args_pair.as_rule() != Rule::function_arguments {
        return Err(format!(
            "Expected function_arguments, got {:?}",
            args_pair.as_rule()
        ));
    }

    let mut arguments = Vec::new();
    for arg_expr_pair in args_pair.into_inner() {
        arguments.push(parse_expression(arg_expr_pair)?);
    }

    Ok(FunctionCall {
        function_name,
        arguments,
        span: fn_call_span,
    })
}

fn expect_rule(pair: &Pair<'_, Rule>, rule: Rule) -> Result<(), String> {
    if pair.as_rule() != rule {
        return Err(format!("Expected {:?}, got {:?}", rule, pair.as_rule()));
    } else {
        Ok(())
    }
}

fn parse_function_definition<'a>(pair: Pair<'a, Rule>) -> Result<FunctionDefinition<'a>, String> {
    expect_rule(&pair, Rule::function_definition)?;
    let fn_def_span = pair.as_span();
    let mut inner_pairs = pair.into_inner();

    let mut parameters = Vec::new();
    // Peek at the next rule. If it's an ident_list, parse parameters.
    // Otherwise, it's the body expression (for functions with no params).
    if let Some(next_pair) = inner_pairs.peek() {
        if next_pair.as_rule() == Rule::ident_list {
            let params_list_pair = inner_pairs.next().unwrap(); // Consume it
            for ident_pair in params_list_pair.into_inner() {
                parameters.push(parse_identifier(ident_pair)?);
            }
        }
    }

    // The next pair must be the body expression
    let body_expr_pair =
        next_inner_or_err(&mut inner_pairs, "function_definition body expression")?;
    let body = parse_expression(body_expr_pair)?;

    Ok(FunctionDefinition {
        parameters,
        body: Box::new(body),
        span: fn_def_span,
    })
}

fn parse_block<'a>(pair: Pair<'a, Rule>) -> Result<Block<'a>, String> {
    if pair.as_rule() != Rule::block {
        return Err(format!(
            "Expected block, got {:?} for \"{}\"",
            pair.as_rule(),
            pair.as_str()
        ));
    }
    let block_span = pair.as_span();
    let mut inner_pairs = pair.into_inner().peekable();
    let mut assignments = Vec::new();

    while let Some(peeked_pair) = inner_pairs.peek() {
        if peeked_pair.as_rule() == Rule::assignment {
            let assignment_pair = inner_pairs.next().unwrap(); // Consume it
            assignments.push(parse_assignment(assignment_pair)?);
        } else {
            break; // Next should be the expression
        }
    }

    let expression_pair = inner_pairs
        .next()
        .ok_or_else(|| "Block: expected expression after assignments".to_string())?;
    let expression = parse_expression(expression_pair)?;

    if inner_pairs.next().is_some() {
        return Err("Block: unexpected extra pairs after expression".to_string());
    }

    Ok(Block {
        assignments,
        expression: Box::new(expression),
        span: block_span,
    })
}

pub fn parse_expression<'a>(pair: Pair<'a, Rule>) -> Result<Expression<'a>, String> {
    if pair.as_rule() != Rule::expression {
        return Err(format!(
            "Expected expression, got {:?} for \"{}\"",
            pair.as_rule(),
            pair.as_str()
        ));
    }
    // An 'expression' rule always contains exactly one inner specific expression type.
    let inner_expr_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| "Expression rule was unexpectedly empty".to_string())?;

    match inner_expr_pair.as_rule() {
        Rule::number => Ok(Expression::Number(parse_number(inner_expr_pair)?)),
        Rule::identifier => Ok(Expression::Identifier(parse_identifier(inner_expr_pair)?)),
        Rule::function_call => Ok(Expression::FunctionCall(parse_function_call(
            inner_expr_pair,
        )?)),
        Rule::function_definition => Ok(Expression::FunctionDefinition(parse_function_definition(
            inner_expr_pair,
        )?)),
        Rule::block => Ok(Expression::Block(parse_block(inner_expr_pair)?)),
        _ => Err(format!(
            "Unexpected rule {:?} inside expression for \"{}\"",
            inner_expr_pair.as_rule(),
            inner_expr_pair.as_str()
        )),
    }
}

fn parse_assignment<'a>(pair: Pair<'a, Rule>) -> Result<Assignment<'a>, String> {
    if pair.as_rule() != Rule::assignment {
        return Err(format!(
            "Expected assignment, got {:?} for \"{}\"",
            pair.as_rule(),
            pair.as_str()
        ));
    }
    let assignment_span = pair.as_span();
    let mut inner_pairs = pair.into_inner();

    let ident_pair = next_inner_or_err(inner_pairs.by_ref(), "assignment identifier")?;
    let identifier = parse_identifier(ident_pair)?;

    let expr_pair = next_inner_or_err(inner_pairs.by_ref(), "assignment expression")?;
    let expression = parse_expression(expr_pair)?;

    Ok(Assignment {
        identifier,
        expression: Box::new(expression),
        span: assignment_span,
    })
}

pub fn parse_program<'a>(pairs: Pairs<'a, Rule>) -> Result<Program<'a>, String> {
    let mut assignments = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::assignment {
            assignments.push(parse_assignment(pair)?);
        } else if pair.as_rule() == Rule::EOI {
        } else {
            // This case should ideally not be reached if grammar is correct
            // and pest only provides significant inner rules.
            return Err(format!(
                "Unexpected rule {:?} inside program for \"{}\"",
                pair.as_rule(),
                pair.as_str()
            ));
        }
    }

    Ok(Program {
        assignments,
        span: Span::new("", 0, 0).unwrap(),
    })
}
