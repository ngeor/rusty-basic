use crate::pc::*;
use crate::pc_specific::*;
use rusty_common::*;

/// Comma separated list of items.
pub fn csv<L: Parser>(parser: L) -> impl Parser<Output = Vec<L::Output>> {
    delimited_by(parser, comma(), trailing_comma_error())
}

pub fn csv_non_opt<P: Parser>(
    parser: P,
    err: &str,
) -> impl Parser<Output = Vec<P::Output>> + NonOptParser {
    csv(parser).or_syntax_error(err)
}

pub fn trailing_comma_error() -> QError {
    QError::syntax_error("Error: trailing comma")
}
