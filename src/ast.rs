use pest::{
    Span,
    iterators::{Pair, Pairs},
};
use std::collections::HashMap;

use crate::parser::Rule;

use cranelift::codegen::ir::{AbiParam, Function, Signature, UserFuncName, InstBuilder, Value, types};
use cranelift::codegen::isa::CallConv;
use cranelift::codegen::{Context, settings};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::prelude::Configurable; // For settings builder
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

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
        span: Span::new("", 0, 0).unwrap(), // TODO: Calculate a proper span for the program
    })
}

// --- Cranelift Compilation ---

fn compile_expression_to_value<'a, 'b>(
    expr: &Expression<'a>,
    builder: &mut FunctionBuilder<'b>,
    module: &mut ObjectModule,
    func_params_map: &HashMap<String, Value>, // Maps AST parameter names to Cranelift values
    // diagnostics_span: Span, // For better error reporting if needed
) -> Result<Value, String> {
    match expr {
        Expression::Number(num) => {
            Ok(builder.ins().iconst(types::I64, num.value))
        }
        Expression::Identifier(ident) => {
            func_params_map.get(&ident.name).copied().ok_or_else(|| {
                format!(
                    "Undefined identifier '{}' used as value at {:?}",
                    ident.name, ident.span
                )
            })
        }
        Expression::FunctionCall(call) => {
            // MVP: Only 'add' function is supported for now
            if call.function_name.name == "add" {
                if call.arguments.len() != 2 {
                    return Err(format!(
                        "'add' function expects 2 arguments, got {} at {:?}",
                        call.arguments.len(),
                        call.span
                    ));
                }
                let arg0_val = compile_expression_to_value(
                    &call.arguments[0],
                    builder,
                    module,
                    func_params_map,
                    // call.arguments[0]. // Need a way to get span from Expression enum
                )?;
                let arg1_val = compile_expression_to_value(
                    &call.arguments[1],
                    builder,
                    module,
                    func_params_map,
                    // call.arguments[1]. // Need a way to get span from Expression enum
                )?;

                let mut sig_add = Signature::new(CallConv::SystemV);
                sig_add.params.push(AbiParam::new(types::I64));
                sig_add.params.push(AbiParam::new(types::I64));
                sig_add.returns.push(AbiParam::new(types::I64));

                let callee_add_id = module
                    .declare_function("add", Linkage::Import, &sig_add)
                    .map_err(|e| format!("Failed to declare 'add' function: {}", e))?;
                let local_callee_add =
                    module.declare_func_in_func(callee_add_id, builder.func);

                let call_inst = builder.ins().call(local_callee_add, &[arg0_val, arg1_val]);
                Ok(builder.inst_results(call_inst)[0])
            } else {
                Err(format!(
                    "Unsupported function call to '{}' at {:?}",
                    call.function_name.name, call.span
                ))
            }
        }
        Expression::Block(block) => {
            // For MVP, blocks in `program/basic.rb`'s main function body don't have assignments.
            // We only compile the final expression of the block.
            if !block.assignments.is_empty() {
                return Err(format!(
                    "Assignments within blocks are not yet supported for compilation at {:?}",
                    block.span
                ));
            }
            compile_expression_to_value(&block.expression, builder, module, func_params_map)
        }
        Expression::FunctionDefinition(fd) => Err(format!(
            "Nested function definitions are not supported for compilation at {:?}",
            fd.span
        )),
    }
}

pub fn compile_program_to_object_bytes(program_ast: &Program) -> Result<Vec<u8>, String> {
    // Find the 'main' function definition
    let main_assignment = program_ast
        .assignments
        .iter()
        .find(|a| a.identifier.name == "main")
        .ok_or_else(|| "No 'main' function assignment found in the program".to_string())?;

    let main_fn_def = match &*main_assignment.expression {
        Expression::FunctionDefinition(fd) => fd,
        _ => return Err(format!("'main' assignment at {:?} is not a function definition", main_assignment.span)),
    };

    if main_fn_def.parameters.len() != 1 {
        return Err(format!(
            "'main' function (at {:?}) must have exactly one parameter, found {}",
            main_fn_def.span,
            main_fn_def.parameters.len()
        ));
    }
    let main_param_name = &main_fn_def.parameters[0].name;

    // Setup Cranelift ISA
    let mut flags_builder = settings::builder();
    flags_builder.set("is_pic", "true").map_err(|e| format!("Failed to set is_pic: {}", e))?;
    let isa_flags = settings::Flags::new(flags_builder);

    let isa_builder = cranelift_native::builder()
        .map_err(|e| format!("Failed to create cranelift_native builder: {}", e))?;
    let isa = isa_builder
        .finish(isa_flags)
        .map_err(|e| format!("Failed to finish ISA building: {}", e))?;

    // Setup Cranelift module
    let mut obj_module = ObjectModule::new(
        ObjectBuilder::new(
            isa.clone(),
            "tempo_module", // Arbitrary internal name for the object module
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| format!("ObjectBuilder error: {}", e))?,
    );

    // Define signature for 'tempo_entry' (our compiled 'main')
    // extern "C" int64_t tempo_entry(int64_t arg);
    let mut sig_tempo_entry = Signature::new(CallConv::SystemV);
    sig_tempo_entry.params.push(AbiParam::new(types::I64)); // input
    sig_tempo_entry.returns.push(AbiParam::new(types::I64)); // output

    let mut func = Function::with_name_signature(UserFuncName::default(), sig_tempo_entry.clone());
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block); // Seal block now that params are appended

    // Map AST parameter name to Cranelift block parameter value
    let mut func_params_map = HashMap::new();
    let clif_param_val = builder.block_params(entry_block)[0];
    func_params_map.insert(main_param_name.clone(), clif_param_val);

    // Compile the body of the main function
    let return_value = compile_expression_to_value(
        &main_fn_def.body,
        &mut builder,
        &mut obj_module,
        &func_params_map,
        // main_fn_def.body.span(), // Need a way to get span from Expression enum
    )?;

    builder.ins().return_(&[return_value]);
    builder.finalize();

    // Declare and define 'tempo_entry'
    let func_id_tempo_entry = obj_module
        .declare_function("tempo_entry", Linkage::Export, &sig_tempo_entry)
        .map_err(|e| format!("Failed to declare 'tempo_entry': {}", e))?;

    let mut ctx = Context::for_function(func);
    obj_module
        .define_function(func_id_tempo_entry, &mut ctx)
        .map_err(|e| format!("Failed to define 'tempo_entry': {}", e))?;

    // Emit object code
    let product = obj_module.finish();
    product
        .emit()
        .map_err(|e| format!("Failed to emit object code: {}", e))
}
