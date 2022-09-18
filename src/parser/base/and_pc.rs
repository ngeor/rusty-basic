//! Contains parser combinators where both parts must succeed.

use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::{Token, Tokenizer};

pub struct TokenParserAndParser<L, R>(L, R)
where
    L: Parser<Output = Token>;

impl<L, R> HasOutput for TokenParserAndParser<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    type Output = (Token, R::Output);
}

impl<L, R> Parser for TokenParserAndParser<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => match self.1.parse(tokenizer)? {
                Some(right) => Ok(Some((left, right))),
                None => {
                    tokenizer.unread(left);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}

pub fn token_parser_and_parser<L, R>(left: L, right: R) -> impl Parser<Output = (Token, R::Output)>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    TokenParserAndParser(left, right)
}

pub trait TokenParserAndParserTrait<P> {
    fn token_and(self, other: P) -> TokenParserAndParser<Self, P>
    where
        Self: Sized + Parser<Output = Token>;
}

impl<S, P> TokenParserAndParserTrait<P> for S
where
    S: Parser<Output = Token>,
{
    fn token_and(self, other: P) -> TokenParserAndParser<Self, P> {
        TokenParserAndParser(self, other)
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
