//! In Parenthesis

use rusty_pc::*;

use crate::input::RcStringView;
use crate::specific::pc_specific::{any_token_of, whitespace, TokenType};
use crate::ParseError;

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
    any_token_of(TokenType::LParen).and_opt_tuple(whitespace())
}

fn right_paren() -> impl Parser<RcStringView, Output = (Option<Token>, Token), Error = ParseError> {
    opt_and_tuple(
        whitespace(),
        any_token_of(TokenType::RParen).no_incomplete(),
    )
}
