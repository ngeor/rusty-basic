use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use std::convert::TryFrom;

pub trait TokenTypeMap {
    type Output;
    fn try_map(&self, token_type: TokenType) -> Option<Self::Output>;

    fn parser(self) -> TokenTypeMapParser<Self>
    where
        Self: Sized,
    {
        TokenTypeMapParser(self)
    }
}

pub struct TokenTypeMapParser<P>(P);

impl<P> Parser for TokenTypeMapParser<P>
where
    P: TokenTypeMap,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => {
                let token_type: TokenType = TokenType::try_from(token.kind)?;
                match self.0.try_map(token_type) {
                    Some(result) => Ok(result),
                    None => {
                        tokenizer.unread(token);
                        Err(QError::Incomplete)
                    }
                }
            }
            None => Err(QError::Incomplete),
        }
    }
}
