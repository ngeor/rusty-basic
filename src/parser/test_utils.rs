use super::{
    Block, ConditionalBlock, Expression, ForLoop, IfBlock, Name, Parser, ProgramNode, Statement,
    TopLevelToken, TopLevelTokenNode,
};
use crate::common::{CaseInsensitiveString, StripLocation};
use crate::lexer::LexerError;
use std::fs::File;

pub type Program = Vec<TopLevelToken>;

pub fn parse<T>(input: T) -> Program
where
    T: AsRef<[u8]>,
{
    let mut parser = Parser::from(input);
    parser
        .parse()
        .expect("Could not parse program")
        .strip_location()
}

pub fn parse_program_node<T>(input: T) -> ProgramNode
where
    T: AsRef<[u8]>,
{
    let mut parser = Parser::from(input);
    parser.parse().expect("Could not parse program")
}

/// Parses the given input, expecting that it will fail.
/// Returns the lexer error.
/// Panics if parsing actually succeeded.
pub fn parse_err<T: AsRef<[u8]>>(input: T) -> LexerError {
    let mut parser = Parser::from(input);
    parser.parse().unwrap_err()
}

/// Parses the given input and asserts that it is parsed successfully and that
/// it contains a single top level token node.
///
/// Return the single top level token node of the parsed program.
pub fn parse_single_top_level_token_node<T: AsRef<[u8]>>(input: T) -> TopLevelTokenNode {
    let mut program = parse_program_node(input);
    assert_eq!(1, program.len());
    program.pop().unwrap()
}

pub fn parse_file<S: AsRef<str>>(filename: S) -> Program {
    let file_path = format!("fixtures/{}", filename.as_ref());
    let mut parser = Parser::from(File::open(file_path).expect("Could not read bas file"));
    parser
        .parse()
        .expect("Could not parse program")
        .strip_location()
}

pub fn sub_call<S: AsRef<str>>(name: S, args: Vec<Expression>) -> Statement {
    Statement::SubCall(CaseInsensitiveString::new(name.as_ref().to_string()), args)
}

pub fn top_sub_call<S: AsRef<str>>(name: S, args: Vec<Expression>) -> TopLevelToken {
    TopLevelToken::Statement(sub_call(name, args))
}

pub fn for_loop(
    variable_name: &str,
    lower_bound: i32,
    upper_bound: i32,
    statements: Block,
) -> Statement {
    Statement::ForLoop(ForLoop {
        variable_name: Name::from(variable_name),
        lower_bound: Expression::from(lower_bound),
        upper_bound: Expression::from(upper_bound),
        step: None,
        statements,
        next_counter: None,
    })
}

pub fn top_for_loop(
    variable_name: &str,
    lower_bound: i32,
    upper_bound: i32,
    statements: Block,
) -> TopLevelToken {
    TopLevelToken::Statement(for_loop(
        variable_name,
        lower_bound,
        upper_bound,
        statements,
    ))
}

pub fn new_if_else(condition: Expression, if_block: Block, else_block: Block) -> IfBlock {
    IfBlock {
        if_block: ConditionalBlock::new(condition, if_block),
        else_if_blocks: vec![],
        else_block: Some(else_block),
    }
}
