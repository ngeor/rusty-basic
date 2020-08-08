use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::declaration::try_read_declaration_parameters;
use crate::parser::error::*;
use crate::parser::name;
use crate::parser::statements::parse_statements;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, ParserError> {
    let p = lexer.peek()?;
    if p.is_keyword(Keyword::Function) {
        let pos = lexer.read()?.pos();
        demand_function_implementation(lexer).map(|x| Some(x.at(pos)))
    } else if p.is_keyword(Keyword::Sub) {
        let pos = lexer.read()?.pos();
        demand_sub_implementation(lexer).map(|x| Some(x.at(pos)))
    } else {
        Ok(None)
    }
}

pub fn demand_function_implementation<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<TopLevelToken, ParserError> {
    // function name
    read_demand_whitespace(lexer, "Expected whitespace after FUNCTION keyword")?;
    let name = demand(lexer, name::try_read, "Expected function name")?;
    // function parameters
    let params: DeclaredNameNodes = demand(
        lexer,
        try_read_declaration_parameters,
        "Expected function parameters",
    )?;
    // function body
    let block = parse_statements(
        lexer,
        |x| x.is_keyword(Keyword::End),
        "Function without End",
    )?;
    read_demand_keyword(lexer, Keyword::End)?;
    read_demand_whitespace(lexer, "Expected whitespace after END keyword")?;
    read_demand_keyword(lexer, Keyword::Function)?;
    Ok(TopLevelToken::FunctionImplementation(name, params, block))
}

pub fn demand_sub_implementation<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<TopLevelToken, ParserError> {
    // sub name
    read_demand_whitespace(lexer, "Expected whitespace after SUB keyword")?;
    let name = demand(lexer, name::try_read_bare, "Expected sub name")?;
    // sub parameters
    let params: DeclaredNameNodes = demand(
        lexer,
        try_read_declaration_parameters,
        "Expected sub parameters",
    )?;
    // body
    let block = parse_statements(lexer, |x| x.is_keyword(Keyword::End), "Sub without End")?;
    read_demand_keyword(lexer, Keyword::End)?;
    read_demand_whitespace(lexer, "Expected whitespace after END keyword")?;
    read_demand_keyword(lexer, Keyword::Sub)?;
    Ok(TopLevelToken::SubImplementation(name, params, block))
}
