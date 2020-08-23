use crate::char_reader::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::declaration;
use crate::parser::name;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    or_ng(function_implementation(), sub_implementation())
}

pub fn function_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    map_ng(
        if_first_demand_second(
            declaration::function_declaration(),
            if_first_demand_second(
                statements::statements(try_read_keyword(Keyword::End)),
                with_keyword(Keyword::End, demand_keyword(Keyword::Function)),
                || QError::SyntaxError("Expected END FUNCTION after function body".to_string()),
            ),
            || QError::SyntaxError("Expected function body".to_string()),
        ),
        |((n, p), (body, _))| TopLevelToken::FunctionImplementation(n, p, body),
    )
}

pub fn sub_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    map_ng(
        if_first_demand_second(
            declaration::sub_declaration(),
            if_first_demand_second(
                statements::statements(try_read_keyword(Keyword::End)),
                with_keyword(Keyword::End, demand_keyword(Keyword::Sub)),
                || QError::SyntaxError("Expected END SUB after sub body".to_string()),
            ),
            || QError::SyntaxError("Expected sub body".to_string()),
        ),
        |((n, p), (body, _))| TopLevelToken::SubImplementation(n, p, body),
    )
}

#[deprecated]
pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, QErrorNode> {
    let p = lexer.peek_ref_dp();
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

#[deprecated]
pub fn demand_function_implementation<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<TopLevelToken, QErrorNode> {
    // function name
    read_whitespace(lexer, "Expected whitespace after FUNCTION keyword")?;
    let name = read(lexer, name::try_read, "Expected function name")?;
    // function parameters
    let params: DeclaredNameNodes = read(
        lexer,
        declaration::try_read_declaration_parameters,
        "Expected function parameters",
    )?;
    // function body
    let block = statements::parse_statements(
        lexer,
        |x| x.is_keyword(Keyword::End),
        "Function without End",
    )?;
    read_keyword(lexer, Keyword::End)?;
    read_whitespace(lexer, "Expected whitespace after END keyword")?;
    read_keyword(lexer, Keyword::Function)?;
    Ok(TopLevelToken::FunctionImplementation(name, params, block))
}

pub fn demand_sub_implementation<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<TopLevelToken, QErrorNode> {
    // sub name
    read_whitespace(lexer, "Expected whitespace after SUB keyword")?;
    let name = read(lexer, name::try_read_bare, "Expected sub name")?;
    // sub parameters
    let params: DeclaredNameNodes = read(
        lexer,
        declaration::try_read_declaration_parameters,
        "Expected sub parameters",
    )?;
    // body
    let block =
        statements::parse_statements(lexer, |x| x.is_keyword(Keyword::End), "Sub without End")?;
    read_keyword(lexer, Keyword::End)?;
    read_whitespace(lexer, "Expected whitespace after END keyword")?;
    read_keyword(lexer, Keyword::Sub)?;
    Ok(TopLevelToken::SubImplementation(name, params, block))
}
