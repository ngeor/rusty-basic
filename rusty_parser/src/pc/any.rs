use crate::pc::{Parser, Token, Tokenizer};
use crate::ParseError;

/// Parses any token.
pub fn any_token() -> AnyTokenParser {
    AnyTokenParser
}

/// Parses any token.
pub struct AnyTokenParser;

impl Parser for AnyTokenParser {
    type Output = Token;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        match tokenizer.read() {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(ParseError::Incomplete),
            Err(err) => Err(err.into()),
        }
    }
}
