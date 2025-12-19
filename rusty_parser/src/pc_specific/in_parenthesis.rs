//! In Parenthesis

use crate::pc::*;
use crate::pc_specific::{any_token_of, whitespace, TokenType};

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<P>(
    parser: P,
) -> impl Parser<RcStringView, Output = <P as Parser<RcStringView>>::Output>
where
    P: Parser<RcStringView>,
{
    parser.surround(left_paren(), right_paren())
}

fn left_paren() -> impl Parser<RcStringView, Output = (Token, Option<Token>)> {
    any_token_of(TokenType::LParen).and_opt_tuple(whitespace())
}

fn right_paren() -> impl Parser<RcStringView, Output = (Option<Token>, Token)> {
    OptAndPC::new(
        whitespace(),
        any_token_of(TokenType::RParen).no_incomplete(),
    )
}
