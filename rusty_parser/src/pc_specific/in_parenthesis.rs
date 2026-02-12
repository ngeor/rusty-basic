//! In Parenthesis

use rusty_pc::*;

use crate::ParserError;
use crate::input::StringView;
use crate::pc_specific::padded_by_ws;
use crate::tokens::{any_symbol_of, any_token_of};

/// Parses the given parser around parenthesis and optional whitespace (inside the parenthesis).
/// If the left side parenthesis is missing, parsing fails (soft).
/// If the right side parenthesis is missing, parsing fails fatally.
pub fn in_parenthesis<P>(
    parser: P,
) -> impl Parser<StringView, Output = P::Output, Error = ParserError>
where
    P: Parser<StringView, Error = ParserError>,
{
    surround(
        left_paren(),
        padded_by_ws(parser),
        right_paren(),
        SurroundMode::Mandatory,
    )
}

fn left_paren() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    any_symbol_of!('(')
}

fn right_paren() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    any_symbol_of!(')')
}
