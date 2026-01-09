use rusty_pc::{ParseResult, Parser, Token, default_parse_error};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::TokenType;

/// Parses any character and returns it as a Symbol token.
pub struct AnySymbolParser;

impl Parser<RcStringView> for AnySymbolParser {
    type Output = Token;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, Self::Output, Self::Error> {
        if input.is_eof() {
            default_parse_error(input)
        } else {
            let ch = input.char();
            Ok((
                input.inc_position(),
                Token::new(TokenType::Symbol.get_index(), ch.to_string()),
            ))
        }
    }
}
