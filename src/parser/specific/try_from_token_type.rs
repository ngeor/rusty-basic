use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::specific::TokenType;
use std::convert::TryFrom;
use std::marker::PhantomData;

pub struct TryFromParser<O>(O)
where
    O: TryFrom<TokenType>;

impl<O> TryFromParser<O>
where
    O: TryFrom<TokenType>,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<O> HasOutput for TryFromParser<O>
where
    O: TryFrom<TokenType>,
{
    type Output = O;
}

impl<O> Parser for TryFromParser<O>
where
    O: TryFrom<TokenType>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => match O::try_from(token.kind as TokenType) {
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
