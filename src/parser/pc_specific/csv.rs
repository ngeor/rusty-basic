use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;

/// Comma separated list of items.
pub fn csv<L: Parser>(parser: L) -> impl Parser<Output = Vec<L::Output>> {
    delimited_by(parser, comma(), trailing_comma_error())
}

pub fn trailing_comma_error() -> QError {
    QError::syntax_error("Error: trailing comma")
}
