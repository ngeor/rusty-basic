//
// TokenPredicate
//

use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Token, Tokenizer};

pub trait TokenPredicate
where
    Self: Sized,
{
    fn test(&self, token: &Token) -> bool;

    fn parser(self) -> TokenPredicateParser<Self> {
        TokenPredicateParser(self)
    }
}

pub trait ErrorProvider {
    fn provide_error(&self) -> QError;
}

pub struct TokenPredicateParser<P>(P);

impl<P> TokenPredicateParser<P> {
    pub fn new(predicate: P) -> Self {
        Self(predicate)
    }
}

impl<P> ParserBase for TokenPredicateParser<P> {
    type Output = Token;
}

impl<P> OptParser for TokenPredicateParser<P>
where
    P: TokenPredicate,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.0.test(&token) {
                    Ok(Some(token))
                } else {
                    tokenizer.unread(token);
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

impl<P> NonOptParser for TokenPredicateParser<P>
where
    P: TokenPredicate + ErrorProvider,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.0.test(&token) {
                    Ok(token)
                } else {
                    Err(self.0.provide_error())
                }
            }
            _ => Err(self.0.provide_error()),
        }
    }
}
