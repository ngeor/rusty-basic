//! Mappers that are able to return an error

use crate::common::QError;
use crate::parser::base::*;

pub struct AndThen<P, F>(P, F);

impl<P, F, U> HasOutput for AndThen<P, F>
where
    P: HasOutput,
    F: Fn(P::Output) -> Result<U, QError>,
{
    type Output = U;
}

impl<P, F, U> Parser for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => (self.1)(value).map(Some),
            None => Ok(None),
        }
    }
}

impl<P, F, U> NonOptParser for AndThen<P, F>
where
    P: NonOptParser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer).and_then(&self.1)
    }
}

pub trait AndThenTrait<F>
where
    Self: Sized,
{
    fn and_then(self, mapper: F) -> AndThen<Self, F>;
}

impl<P, F, U> AndThenTrait<F> for P
where
    P: HasOutput,
    F: Fn(P::Output) -> Result<U, QError>,
{
    fn and_then(self, mapper: F) -> AndThen<Self, F> {
        AndThen(self, mapper)
    }
}
