use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::specific::TokenType;
use std::convert::TryFrom;

pub trait TokenTypeMap: HasOutput {
    fn try_map(&self, token_type: TokenType) -> Option<Self::Output>;

    fn parser(self) -> TokenTypeMapParser<Self>
    where
        Self: Sized,
    {
        TokenTypeMapParser(self)
    }
}

pub struct TokenTypeMapParser<P>(P);

impl<P> HasOutput for TokenTypeMapParser<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<P> Parser for TokenTypeMapParser<P>
where
    P: TokenTypeMap,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                let token_type: TokenType = TokenType::try_from(token.kind)?;
                match self.0.try_map(token_type) {
                    Some(result) => Ok(Some(result)),
                    None => {
                        tokenizer.unread(token);
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }
}
