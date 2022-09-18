//! Contains parser combinators where both parts must succeed.

use crate::common::QError;
use crate::parser::base::parsers::{NonOptParser, NonOptParserResult, Parser, ParserResult};
use crate::parser::base::tokenizers::{Token, Tokenizer};

pub struct TokenParserAndParser<L, R>(L, R)
where
    L: Parser<Output = Token>;

impl<L, R> Parser for TokenParserAndParser<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    type Output = (Token, R::Output);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> ParserResult<Self::Output> {
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

pub struct AndPC<L, R>(L, R);

impl<L, R> Parser for AndPC<L, R>
where
    L: Parser,
    R: NonOptParser,
{
    type Output = (L::Output, R::Output);

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

impl<L, R> NonOptParser for AndPC<L, R>
where
    L: NonOptParser,
    R: NonOptParser,
{
    type Output = (L::Output, R::Output);

    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> NonOptParserResult<Self::Output> {
        let left = self.0.parse_non_opt(tokenizer)?;
        let right = self.1.parse_non_opt(tokenizer)?;
        Ok((left, right))
    }
}

pub trait AndTrait<P> {
    fn and(self, other: P) -> AndPC<Self, P>
    where
        Self: Sized;
}

impl<S, P> AndTrait<P> for S {
    fn and(self, other: P) -> AndPC<Self, P> {
        AndPC(self, other)
    }
}

// TODO check if we can keep just the AddTrait as this is now the same

pub trait AndDemandTrait<P>
where
    Self: Sized,
    P: NonOptParser,
{
    fn and_demand(self, other: P) -> AndPC<Self, P>;
}

impl<S, P> AndDemandTrait<P> for S
where
    S: Parser,
    P: NonOptParser,
{
    fn and_demand(self, other: P) -> AndPC<Self, P> {
        AndPC(self, other)
    }
}

// impl<S, P> AndDemandTrait<P> for S
// where
//     S: NonOptParser,
//     P: NonOptParser,
// {
//     fn and_demand(self, other: P) -> AndPC<Self, P> {
//         AndPC(self, other)
//     }
// }
