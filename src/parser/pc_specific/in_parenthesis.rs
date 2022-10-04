//! In Parenthesis

use crate::parser::pc::*;
use crate::parser::pc_specific::{any_token_of, TokenType};

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<P>(parser: P) -> impl Parser<Output = <P as Parser>::Output>
where
    P: Parser + NonOptParser,
{
    seq3(left_paren(), parser, right_paren(), |_, value, _| value)
}

fn left_paren() -> impl Parser<Output = Token> {
    any_token_of(TokenType::LParen)
}

fn right_paren() -> impl Parser<Output = Token> + NonOptParser {
    any_token_of(TokenType::RParen).no_incomplete()
}
