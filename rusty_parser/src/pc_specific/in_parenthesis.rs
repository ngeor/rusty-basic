//! In Parenthesis

use crate::pc::*;
use crate::pc_specific::{any_token_of, whitespace, TokenType};

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<I: Tokenizer + 'static, P>(
    parser: P,
) -> impl Parser<I, Output = <P as Parser<I>>::Output>
where
    P: Parser<I>,
{
    parser.surround(left_paren(), right_paren())
}

fn left_paren<I: Tokenizer + 'static>() -> impl Parser<I, Output = (Token, Option<Token>)> {
    any_token_of(TokenType::LParen).and_opt_tuple(whitespace())
}

fn right_paren<I: Tokenizer + 'static>() -> impl Parser<I, Output = (Option<Token>, Token)> {
    OptAndPC::new(
        whitespace(),
        any_token_of(TokenType::RParen).no_incomplete(),
    )
}
