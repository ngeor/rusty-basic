use crate::pc::{Parser, Token, Tokenizer};
use crate::ParseError;

/// Parses any token.
pub fn any_token<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    AnyTokenParser
}

/// Parses any token.
pub struct AnyTokenParser;

impl<I: Tokenizer + 'static> Parser<I> for AnyTokenParser {
    type Output = Token;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match tokenizer.read() {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(ParseError::Incomplete),
            Err(err) => Err(err.into()),
        }
    }
}
