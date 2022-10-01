use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;

/// Comma separated list of items.
/// When used as a parser, returns one or more items.
/// When used as a non-opt-parser, returns zero or more items.
pub fn csv<L: Parser>(parser: L, allow_empty: bool) -> impl Parser<Output = Vec<L::Output>> {
    delimited_by(parser, comma(), allow_empty, trailing_comma_error())
}

pub fn trailing_comma_error() -> QError {
    QError::syntax_error("Error: trailing comma")
}
