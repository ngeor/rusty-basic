use crate::char_reader::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::declaration;
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
                with_keyword_before(Keyword::End, demand_keyword(Keyword::Function)),
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
                with_keyword_before(Keyword::End, demand_keyword(Keyword::Sub)),
                || QError::SyntaxError("Expected END SUB after sub body".to_string()),
            ),
            || QError::SyntaxError("Expected sub body".to_string()),
        ),
        |((n, p), (body, _))| TopLevelToken::SubImplementation(n, p, body),
    )
}
