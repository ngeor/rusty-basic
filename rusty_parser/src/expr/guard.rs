use rusty_pc::*;

use crate::ParserError;
use crate::input::StringView;
use crate::pc_specific::{WithExpected, whitespace_ignoring};
use crate::tokens::{any_symbol_of, any_token_of};

/// `result ::= " " | "("`
///
/// The "(" will be undone.
pub fn parser() -> impl Parser<StringView, Output = (), Error = ParserError> {
    whitespace_guard()
        .or(lparen_guard())
        .with_expected_message("Expected: '(' or whitespace")
}

fn whitespace_guard() -> impl Parser<StringView, Output = (), Error = ParserError> {
    whitespace_ignoring()
}

fn lparen_guard() -> impl Parser<StringView, Output = (), Error = ParserError> {
    any_symbol_of!('(').map_to_unit().peek()
}
