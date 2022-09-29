use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use std::convert::TryFrom;

pub trait TokenTypeMap: ParserBase {
    fn try_map(&self, token_type: TokenType) -> Option<Self::Output>;

    fn parser(self) -> TokenTypeMapParser<Self>
    where
        Self: Sized,
    {
        TokenTypeMapParser(self)
    }
}

pub struct TokenTypeMapParser<P>(P);

impl<P> ParserBase for TokenTypeMapParser<P>
where
    P: ParserBase,
{
    type Output = P::Output;
}

impl<P> OptParser for TokenTypeMapParser<P>
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
