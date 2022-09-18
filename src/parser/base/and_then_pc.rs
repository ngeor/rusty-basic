//! Mappers that are able to return an error

use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::Tokenizer;

pub struct AndThen<P, F>(P, F)
where
    P: Parser;

impl<P, F, U> HasOutput for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<Option<U>, QError>,
{
    type Output = U;
}

impl<P, F, U> Parser for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<Option<U>, QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => (self.1)(value),
            None => Ok(None),
        }
    }
}

pub trait AndThenTrait<F> {
    fn and_then(self, mapper: F) -> AndThen<Self, F>
    where
        Self: Sized + Parser;
}

impl<P, F, U> AndThenTrait<F> for P
where
    P: Parser,
    F: Fn(P::Output) -> Result<Option<U>, QError>,
{
    fn and_then(self, mapper: F) -> AndThen<Self, F> {
        AndThen(self, mapper)
    }
}
