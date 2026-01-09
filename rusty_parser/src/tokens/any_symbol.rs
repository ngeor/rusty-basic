use rusty_pc::{Map, Parser, Token};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::TokenType;
use crate::tokens::any_char::AnyChar;

/// Parses any character and returns it as a Symbol token.
pub(super) fn any_symbol() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    AnyChar.map(|ch| Token::new(TokenType::Symbol.get_index(), ch.to_string()))
}
