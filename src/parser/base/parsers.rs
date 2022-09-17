use super::tokenizers::{Token, Tokenizer};
use crate::common::QError;

/// A parser that either succeeds or returns an error.
pub trait NonOptParser {
    type Output;
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError>;
}

// TODO rename to OptParser
/// A parser that either succeeds, or returns nothing, or returns an error.
pub trait Parser {
    type Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError>;
}

//
// TokenPredicate
//

pub trait TokenPredicate {
    fn test(&self, token: &Token) -> bool;
}

pub trait ErrorProvider {
    fn provide_error(&self) -> QError;
}

impl<T> Parser for T
where
    T: TokenPredicate,
{
    type Output = Token;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.test(&token) {
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

impl<T> NonOptParser for T
where
    T: TokenPredicate + ErrorProvider,
{
    type Output = Token;

    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.test(&token) {
                    Ok(token)
                } else {
                    Err(self.provide_error())
                }
            }
            _ => Err(self.provide_error()),
        }
    }
}

//
// AndThenMapper
//

pub struct AndThenMapper<P, F>(P, F);

impl<P, F, U> Parser for AndThenMapper<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<Option<U>, QError>,
{
    type Output = U;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => (self.1)(value),
            None => Ok(None),
        }
    }
}

impl<P, F, U> NonOptParser for AndThenMapper<P, F>
where
    P: NonOptParser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    type Output = U;

    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0
            .parse_non_opt(tokenizer)
            .and_then(|value| (self.1)(value))
    }
}

pub trait AndThenTrait<F> {
    fn and_then(self, mapper: F) -> AndThenMapper<Self, F>;
}

impl<S, F, U> AndThenTrait<F> for S
where
    S: Parser,
    F: Fn(S::Output) -> Result<U, QError>,
{
    fn and_then(self, mapper: F) -> AndThenMapper<Self, F> {
        AndThenMapper(self, mapper)
    }
}

//
// The left side can be followed by an optional right.
//

pub struct AndOptPC<L, R>(L, R);

impl<L, R> Parser for AndOptPC<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => {
                let opt_right = self.1.parse(tokenizer)?;
                Ok(Some((left, opt_right)))
            }
            None => Ok(None),
        }
    }
}

pub trait AndOptTrait<P> {
    fn and_opt(self, other: P) -> AndOptPC<Self, P>;
}

impl<S, P> AndOptTrait<P> for S
where
    S: Parser,
    P: Parser,
{
    fn and_opt(self, other: P) -> AndOptPC<Self, P> {
        AndOptPC(self, other)
    }
}

//
// The left side is optional, the right is not.
// If the right is missing, the left is reverted.
//

pub struct OptAndPC<L, R>(L, R);

impl<L, R> OptAndPC<L, R> {
    pub fn new(left : L, right: R) -> Self {
        Self(left, right)
    }
}

impl<L, R> Parser for OptAndPC<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    type Output = (Option<Token>, R::Output);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_leading = self.0.parse(tokenizer)?;
        match self.1.parse(tokenizer)? {
            Some(value) => Ok(Some((opt_leading, value))),
            None => {
                if let Some(token) = opt_leading {
                    tokenizer.unread(token);
                }
                Ok(None)
            }
        }
    }
}

//
// Both parts must succeed.
//

pub struct AndPC<L, R>(L, R);

impl<L, R> Parser for AndPC<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
    type Output = (Token, R::Output);

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

    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.0.parse_non_opt(tokenizer)?;
        let right = self.1.parse_non_opt(tokenizer)?;
        Ok((left, right))
    }
}

pub trait AndTrait<P> {
    fn and(self, other: P) -> AndPC<Self, P>;
}

impl<S, P> AndTrait<P> for S
where
    S: Parser<Output = Token>,
    P: Parser,
{
    fn and(self, other: P) -> AndPC<Self, P> {
        AndPC(self, other)
    }
}

pub trait AndDemandTrait<P> {
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

impl<S, P> AndDemandTrait<P> for S
where
    S: NonOptParser,
    P: NonOptParser,
{
    fn and_demand(self, other: P) -> AndPC<Self, P> {
        AndPC(self, other)
    }
}

//
// Keep Left
//

pub struct KeepLeftMapper<P>(P);

impl<P, L, R> Parser for KeepLeftMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    type Output = R;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(l, r)| l))
    }
}

pub trait KeepLeftTrait {
    fn keep_left(self) -> KeepLeftMapper<Self>;
}

impl<S, L, R> KeepLeftTrait for S
where
    S: Parser<Output = (L, R)>,
{
    fn keep_left(self) -> KeepLeftMapper<Self> {
        KeepLeftMapper(self)
    }
}

//
// Keep Right
//

pub struct KeepRightMapper<P>(P);

impl<P, L, R> Parser for KeepRightMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    type Output = R;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(l, r)| r))
    }
}

pub trait KeepRightTrait {
    fn keep_right(self) -> KeepRightMapper<Self>;
}

impl<S, L, R> KeepRightTrait for S
where
    S: Parser<Output = (L, R)>,
{
    fn keep_right(self) -> KeepRightMapper<Self> {
        KeepRightMapper(self)
    }
}

//
// Map
//

pub struct FnMapper<P, F>(P, F);

impl<P, F, U> Parser for FnMapper<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> U,
{
    type Output = U;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(&self.1))
    }
}

pub trait FnMapTrait<F> {
    fn map(self, mapper: F) -> FnMapper<Self, F>;
}

impl<S, F, U> FnMapTrait<F> for S
where
    S: Parser,
    F: Fn(S::Output) -> U,
{
    fn map(self, mapper: F) -> FnMapper<Self, F> {
        FnMapper(self, mapper)
    }
}
