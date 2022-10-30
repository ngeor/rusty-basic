//! In Parenthesis

use crate::pc::*;
use crate::pc_specific::{any_token_of, whitespace, TokenType};

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<P>(parser: P) -> impl Parser<Output = <P as Parser>::Output>
where
    P: Parser + NonOptParser,
{
    seq3(left_paren(), parser, right_paren(), |_, value, _| value)
}

fn left_paren() -> impl Parser<Output = (Token, Option<Token>)> {
    any_token_of(TokenType::LParen).and_opt(whitespace())
}

fn right_paren() -> impl Parser<Output = (Option<Token>, Token)> + NonOptParser {
    OptAndPC::new(
        whitespace(),
        any_token_of(TokenType::RParen).no_incomplete(),
    )
}
