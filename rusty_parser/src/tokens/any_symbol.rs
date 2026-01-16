use rusty_pc::text::{CharInput, any_char};
use rusty_pc::{Map, Parser, Token};

use crate::tokens::TokenType;

/// Parses any character and returns it as a Symbol token.
pub(super) fn any_symbol<I, E>() -> impl Parser<I, Output = Token, Error = E>
where
    I: CharInput,
    E: Default,
{
    any_char().map(|ch| Token::new(TokenType::Symbol.get_index(), ch.to_string()))
}
