use super::{
    Block, ConditionalBlock, Expression, ForLoop, IfBlock, Name, Parser, ProgramNode, Statement,
    TopLevelToken,
};
use crate::common::CaseInsensitiveString;
use std::fs::File;
pub fn parse<T>(input: T) -> ProgramNode
where
    T: AsRef<[u8]>,
{
    let mut parser = Parser::from(input);
    parser.parse().expect("Could not parse program")
}

pub fn parse_file<S: AsRef<str>>(filename: S) -> ProgramNode {
    let file_path = format!("fixtures/{}", filename.as_ref());
    let mut parser = Parser::from(File::open(file_path).expect("Could not read bas file"));
    parser.parse().expect("Could not parse program")
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
