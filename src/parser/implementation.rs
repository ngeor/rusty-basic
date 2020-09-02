use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::declaration;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::misc::*;
use crate::parser::pc::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    or(function_implementation(), sub_implementation())
}

pub fn function_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(
        seq5(
            declaration::function_declaration(),
            statements::statements(
                try_read_keyword(Keyword::End),
                QError::syntax_error_fn("Expected: end-of-statement"),
            ),
            demand(
                try_read_keyword(Keyword::End),
                QError::syntax_error_fn("Expected: END FUNCTION"),
            ),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after END"),
            ),
            demand(
                try_read_keyword(Keyword::Function),
                QError::syntax_error_fn("Expected: FUNCTION after END"),
            ),
        ),
        |((n, p), body, _, _, _)| TopLevelToken::FunctionImplementation(n, p, body),
    )
}

pub fn sub_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(
        seq5(
            declaration::sub_declaration(),
            statements::statements(
                try_read_keyword(Keyword::End),
                QError::syntax_error_fn("Expected: end-of-statement"),
            ),
            demand(
                try_read_keyword(Keyword::End),
                QError::syntax_error_fn("Expected: END SUB"),
            ),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after END"),
            ),
            demand(
                try_read_keyword(Keyword::Sub),
                QError::syntax_error_fn("Expected: SUB after END"),
            ),
        ),
        |((n, p), body, _, _, _)| TopLevelToken::SubImplementation(n, p, body),
    )
}
