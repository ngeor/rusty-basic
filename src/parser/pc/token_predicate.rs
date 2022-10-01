//
// TokenPredicate
//

use crate::common::QError;
use crate::parser::pc::{Parser, Token, Tokenizer};

#[deprecated]
pub trait TokenPredicate
where
    Self: Sized,
{
    fn test(&self, token: &Token) -> bool;

    fn parser(self) -> TokenPredicateParser<Self> {
        TokenPredicateParser(self)
    }
}

#[deprecated]
pub trait ErrorProvider {
    fn provide_error_message(&self) -> String;

    fn to_err<T>(&self) -> Result<T, QError> {
        Err(QError::Expected(self.provide_error_message()))
    }
}

pub struct TokenPredicateParser<P>(P);

impl<P> TokenPredicateParser<P> {
    pub fn new(predicate: P) -> Self {
        Self(predicate)
    }
}

impl<P> Parser for TokenPredicateParser<P>
where
    P: TokenPredicate + ErrorProvider,
{
    type Output = Token;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) if self.0.test(&token) => {
                return Ok(token);
            }
            Some(token) => {
                tokenizer.unread(token);
            }
            None => {}
        }
        self.0.to_err()
    }
}
