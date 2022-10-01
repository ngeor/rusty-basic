use crate::common::QError;
use crate::parser::pc::{Parser, Token, Tokenizer};

/// Parses any token.
pub fn any_token() -> AnyTokenParser {
    AnyTokenParser
}

/// Parses any token.
pub struct AnyTokenParser;

impl Parser for AnyTokenParser {
    type Output = Token;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read() {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(QError::Incomplete),
            Err(err) => Err(err.into()),
        }
    }
}
