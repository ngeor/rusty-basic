//! In Parenthesis

use rusty_pc::*;

use crate::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::SpecificTrait;
use crate::tokens::{any_symbol_of, any_token_of};

/// Parses the given parser around parenthesis and optional whitespace (inside the parenthesis).
/// If the left side parenthesis is missing, parsing fails (non-fatal).
/// If the right side parenthesis is missing, parsing fails fatally.
///
/// # Warning
/// The given parser cannot return a non-fatal result.
pub fn in_parenthesis<P>(
    parser: P,
) -> impl Parser<RcStringView, Output = P::Output, Error = ParseError>
where
    P: Parser<RcStringView, Error = ParseError>,
{
    surround(
        left_paren(),
        parser.padded_by_ws(),
        right_paren(),
        SurroundMode::Mandatory,
    )
}

fn left_paren() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    // TODO add ignoring support for parenthesis
    any_symbol_of!('(')
}

fn right_paren() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    // TODO add ignoring support for parenthesis
    any_symbol_of!(')')
}
