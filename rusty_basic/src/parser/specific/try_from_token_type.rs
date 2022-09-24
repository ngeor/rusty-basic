use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::specific::TokenType;
use std::convert::TryFrom;
use std::marker::PhantomData;
use tokenizers::Tokenizer;

pub struct TryFromParser<O>(PhantomData<O>);

impl<O> TryFromParser<O> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<O> HasOutput for TryFromParser<O> {
    type Output = O;
}

impl<O> Parser for TryFromParser<O>
where
    O: TryFrom<TokenType, Error = QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => match TokenType::try_from(token.kind).and_then(O::try_from) {
                Ok(value) => Ok(Some(value)),
                Err(_) => {
                    tokenizer.unread(token);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}
