use crate::common::QError;
use super::tokenizers::{Token, Tokenizer};

trait Parser {
    type Item;
    fn parse(source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError>;
}

struct AnyTokenParser{}

impl Parser for AnyTokenParser {
    type Item = Token;

    fn parse(source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        source.read().map_err(|e| e.into())
    }
}
