//! Contains parser combinators where both parts must succeed.

use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::base::undo_pc::Undo;

pub struct AndPC<L, R>(L, R);

impl<L, R> HasOutput for AndPC<L, R>
where
    L: HasOutput,
    R: HasOutput,
{
    type Output = (L::Output, R::Output);
}

impl<L, R> Parser for AndPC<L, R>
where
    L: Parser,
    L::Output: Undo,
    R: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => match self.1.parse(tokenizer)? {
                Some(right) => Ok(Some((left, right))),
                None => {
                    left.undo(tokenizer);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}

pub trait AndTrait<P>
where
    Self: Sized,
{
    fn and(self, other: P) -> AndPC<Self, P>;
}

impl<S, P> AndTrait<P> for S
where
    S: Sized,
{
    fn and(self, other: P) -> AndPC<Self, P> {
        AndPC(self, other)
    }
}

pub struct AndDemandPC<L, R>(L, R);

impl<L, R> HasOutput for AndDemandPC<L, R>
where
    L: HasOutput,
    R: HasOutput,
{
    type Output = (L::Output, R::Output);
}

impl<L, R> Parser for AndDemandPC<L, R>
where
    L: Parser,
    R: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => {
                let right = self.1.parse_non_opt(tokenizer)?;
                Ok(Some((left, right)))
            }
            None => Ok(None),
        }
    }
}

impl<L, R> NonOptParser for AndDemandPC<L, R>
where
    L: NonOptParser,
    R: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.0.parse_non_opt(tokenizer)?;
        let right = self.1.parse_non_opt(tokenizer)?;
        Ok((left, right))
    }
}

pub trait AndDemandTrait<P> {
    fn and_demand(self, other: P) -> AndDemandPC<Self, P>
    where
        Self: Sized;
}

impl<S, P> AndDemandTrait<P> for S {
    fn and_demand(self, other: P) -> AndDemandPC<Self, P> {
        AndDemandPC(self, other)
    }
}
