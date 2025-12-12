use crate::pc::{Parser, Token, Tokenizer};
use crate::ParseError;

/// Parses any token.
pub fn any_token<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    AnyTokenParser
}

struct AnyTokenParser;

impl<I: Tokenizer + 'static> Parser<I> for AnyTokenParser {
    type Output = Token;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match tokenizer.read() {
            Some(token) => Ok(token),
            None => Err(ParseError::Incomplete),
        }
    }
}

/// Peeks the next token without consuming it.
pub fn peek_token<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    PeekTokenParser
}

struct PeekTokenParser;

impl<I: Tokenizer + 'static> Parser<I> for PeekTokenParser {
    type Output = Token;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match tokenizer.read() {
            Some(token) => {
                tokenizer.unread(token.clone());
                Ok(token)
            }
            None => Err(ParseError::Incomplete),
        }
    }
}

/// Returns Ok(()) if we're at EOF,
/// otherwise an incomplete result,
/// without modifying the input.
pub fn detect_eof<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    EofDetector
}

struct EofDetector;

impl<I: Tokenizer + 'static> Parser<I> for EofDetector {
    type Output = ();

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match tokenizer.read() {
            Some(token) => {
                tokenizer.unread(token);
                Err(ParseError::Incomplete)
            }
            None => Ok(()),
        }
    }
}
