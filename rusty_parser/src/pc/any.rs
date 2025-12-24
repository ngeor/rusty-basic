use crate::error::ParseError;
use crate::pc::{default_parse_error, ParseResult, Parser, RcStringView, Token};

// TODO: fix this
use crate::specific::token_parser;

/// Parses any token.
pub fn any_token() -> impl Parser<RcStringView, Output = Token> {
    token_parser()
}

/// Peeks the next token without consuming it.
pub fn peek_token() -> impl Parser<RcStringView, Output = Token> {
    PeekTokenParser
}

struct PeekTokenParser;

impl Parser<RcStringView> for PeekTokenParser {
    type Output = Token;

    fn parse(
        &self,
        tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
        match token_parser().parse(tokenizer.clone()) {
            Ok((_, value)) => Ok((tokenizer, value)),
            Err(err) => Err(err),
        }
    }
}

/// Returns Ok(()) if we're at EOF,
/// otherwise an incomplete result,
/// without modifying the input.
pub fn detect_eof() -> impl Parser<RcStringView, Output = ()> {
    EofDetector
}

struct EofDetector;

impl Parser<RcStringView> for EofDetector {
    type Output = ();

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
