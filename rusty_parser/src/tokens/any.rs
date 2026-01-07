use rusty_pc::{ParseResult, Parser, Token, default_parse_error};

use crate::error::ParseError;
use crate::input::RcStringView;
use crate::tokens::any_token;

/// Peeks the next token without consuming it.
pub fn peek_token() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    PeekTokenParser
}

struct PeekTokenParser;

impl Parser<RcStringView> for PeekTokenParser {
    type Output = Token;
    type Error = ParseError;

    fn parse(
        &self,
        tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
        match any_token().parse(tokenizer.clone()) {
            Ok((_, value)) => Ok((tokenizer, value)),
            Err(err) => Err(err),
        }
    }
}

/// Returns Ok(()) if we're at EOF,
/// otherwise an incomplete result,
/// without modifying the input.
pub fn detect_eof() -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    EofDetector
}

struct EofDetector;

impl Parser<RcStringView> for EofDetector {
    type Output = ();
    type Error = ParseError;

    fn parse(
        &self,
        tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
        match peek_token().parse(tokenizer) {
            Ok((i, _)) => default_parse_error(i),
            Err((false, i, _)) => Ok((i, ())),
            Err(err) => Err(err),
        }
    }
}
