use rusty_pc::text::any_char;
use rusty_pc::{InputTrait, Parser, ParserErrorTrait, Token};

use crate::tokens::TokenType;

/// Parses any character and returns it as a Symbol token.
pub(super) fn any_symbol<I, E>() -> impl Parser<I, Output = Token, Error = E>
where
    I: InputTrait<Output = char>,
    E: ParserErrorTrait,
{
    any_char().map(|ch| Token::new(TokenType::Symbol.get_index(), ch.to_string()))
}
