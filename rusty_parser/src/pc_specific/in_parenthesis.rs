//! In Parenthesis

use rusty_pc::*;

use crate::ParserError;
use crate::input::RcStringView;
use crate::pc_specific::PaddedByWs;
use crate::tokens::{any_symbol_of, any_token_of};

/// Parses the given parser around parenthesis and optional whitespace (inside the parenthesis).
/// If the left side parenthesis is missing, parsing fails (soft).
/// If the right side parenthesis is missing, parsing fails fatally.
///
/// # Warning
/// The given parser cannot return a soft error.
pub fn in_parenthesis<P>(
    parser: P,
) -> impl Parser<RcStringView, Output = P::Output, Error = ParserError>
where
    P: Parser<RcStringView, Error = ParserError>,
{
    surround(
        left_paren(),
        parser.padded_by_ws(),
        right_paren(),
        SurroundMode::Mandatory,
    )
}

fn left_paren() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    // TODO add ignoring support for parenthesis
    any_symbol_of!('(')
}

fn right_paren() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    // TODO add ignoring support for parenthesis
    any_symbol_of!(')')
}
