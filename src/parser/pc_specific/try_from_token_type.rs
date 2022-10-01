use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use std::convert::TryFrom;
use std::marker::PhantomData;

pub struct TryFromParser<O>(PhantomData<O>);

impl<O> TryFromParser<O> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<O> ParserBase for TryFromParser<O> {
    type Output = O;
}

impl<O> Parser for TryFromParser<O>
where
    O: TryFrom<TokenType, Error = QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => match TokenType::try_from(token.kind).and_then(O::try_from) {
                Ok(value) => Ok(value),
                Err(_) => {
                    tokenizer.unread(token);
                    Err(QError::Incomplete)
                }
            },
            None => Err(QError::Incomplete),
        }
    }
}
