//! In Parenthesis

use rusty_pc::*;

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::{any_symbol_of, any_token_of, whitespace};

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<P>(
    parser: P,
) -> impl Parser<RcStringView, Output = P::Output, Error = ParseError>
where
    P: Parser<RcStringView, Error = ParseError>,
{
    parser.surround(left_paren(), right_paren())
}

fn left_paren() -> impl Parser<RcStringView, Output = (Token, Option<Token>), Error = ParseError> {
    // TODO add ignoring for whitespace, add ignoring for and_opt_tuple
    any_symbol_of!('(').and_opt_tuple(whitespace())
}

fn right_paren() -> impl Parser<RcStringView, Output = (Option<Token>, Token), Error = ParseError> {
    // TODO add ignoring for whitespace, add ignoring for opt_and_tuple
    opt_and_tuple(whitespace(), any_symbol_of!(')').no_incomplete())
}
