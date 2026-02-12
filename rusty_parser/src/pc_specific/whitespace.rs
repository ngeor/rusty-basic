use rusty_pc::{Parser, SurroundMode, Token, surround};

use crate::ParserError;
use crate::input::StringView;
use crate::tokens::{TokenType, any_token_of};

pub fn whitespace() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    any_token_of!(TokenType::Whitespace)
}

pub fn whitespace_ignoring() -> impl Parser<StringView, Output = (), Error = ParserError> {
    whitespace().map_to_unit()
}

pub trait PaddedByWs: Parser<StringView, Error = ParserError>
where
    Self: Sized,
{
    fn padded_by_ws(self) -> impl Parser<StringView, Output = Self::Output, Error = Self::Error> {
        surround(
            whitespace_ignoring(),
            self,
            whitespace_ignoring(),
            SurroundMode::Optional,
        )
    }
}

impl<P> PaddedByWs for P where P: Parser<StringView, Error = ParserError> {}
