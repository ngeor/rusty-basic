use crate::common::QError;

use super::tokenizers::{Token, Tokenizer};

pub trait HasOutput {
    type Output;
}

/// A parser that either succeeds or returns an error.
pub trait NonOptParser: HasOutput {
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError>;
}

// TODO rename to OptParser
/// A parser that either succeeds, or returns nothing, or returns an error.
pub trait Parser: HasOutput {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError>;
}

pub fn to_impl_parser<P: Parser>(parser: P) -> impl Parser<Output = P::Output> {
    parser
}

// TODO check if possible to inherit from this and use it
pub trait ParserFactory<P> {
    fn parser(self) -> P;
}

//
// TokenPredicate
//

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

impl<P> HasOutput for TokenPredicateParser<P> {
    type Output = Token;
}

impl<P> Parser for TokenPredicateParser<P>
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
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
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

//
// The left side can be followed by an optional right.
//

pub struct AndOptPC<L, R>(L, R);

impl<L, R> HasOutput for AndOptPC<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);
}

impl<L, R> Parser for AndOptPC<L, R>
where
    L: Parser,
    R: Parser,
{
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
    fn and_opt(self, other: P) -> AndOptPC<Self, P>
    where
        Self: Sized;
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

/// The left side is optional, the right is not.
/// If the right is missing, the left is reverted.
pub struct OptAndPC<L, R>(L, R);

impl<L, R> OptAndPC<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<L, R> HasOutput for OptAndPC<L, R>
where
    L: HasOutput<Output = Token>,
    R: HasOutput,
{
    type Output = (Option<Token>, R::Output);
}

impl<L, R> Parser for OptAndPC<L, R>
where
    L: Parser<Output = Token>,
    R: Parser,
{
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
// Keep Left
//

pub struct KeepLeftMapper<P>(P);

impl<P, L, R> HasOutput for KeepLeftMapper<P>
where
    P: HasOutput<Output = (L, R)>,
{
    type Output = L;
}

impl<P, L, R> Parser for KeepLeftMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(l, r)| l))
    }
}

impl<P, L, R> NonOptParser for KeepLeftMapper<P>
where
    P: NonOptParser<Output = (L, R)>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer).map(|(l, r)| l)
    }
}

pub trait KeepLeftTrait {
    fn keep_left(self) -> KeepLeftMapper<Self>
    where
        Self: Sized;
}

impl<S, L, R> KeepLeftTrait for S
where
    S: HasOutput<Output = (L, R)>,
{
    fn keep_left(self) -> KeepLeftMapper<Self> {
        KeepLeftMapper(self)
    }
}

//
// Keep Right
//

pub struct KeepRightMapper<P>(P);

impl<P, L, R> HasOutput for KeepRightMapper<P>
where
    P: HasOutput<Output = (L, R)>,
{
    type Output = R;
}

impl<P, L, R> Parser for KeepRightMapper<P>
where
    P: Parser<Output = (L, R)>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(l, r)| r))
    }
}

impl<P, L, R> NonOptParser for KeepRightMapper<P>
where
    P: NonOptParser<Output = (L, R)>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer).map(|(l, r)| r)
    }
}

pub trait KeepRightTrait {
    fn keep_right(self) -> KeepRightMapper<Self>
    where
        Self: Sized;
}

impl<S, L, R> KeepRightTrait for S
where
    S: HasOutput<Output = (L, R)>,
{
    fn keep_right(self) -> KeepRightMapper<Self> {
        KeepRightMapper(self)
    }
}

//
// Map
//

pub struct FnMapper<P, F>(P, F);

impl<P, F, U> HasOutput for FnMapper<P, F>
where
    P: HasOutput,
    F: Fn(P::Output) -> U,
{
    type Output = U;
}

impl<P, F, U> Parser for FnMapper<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> U,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(&self.1))
    }
}

impl<P, F, U> NonOptParser for FnMapper<P, F>
where
    P: NonOptParser,
    F: Fn(P::Output) -> U,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer).map(&self.1)
    }
}

pub trait FnMapTrait<F> {
    fn fn_map(self, mapper: F) -> FnMapper<Self, F>
    where
        Self: Sized;
}

impl<S, F, U> FnMapTrait<F> for S
where
    S: Parser,
    F: Fn(S::Output) -> U,
{
    fn fn_map(self, mapper: F) -> FnMapper<Self, F> {
        FnMapper(self, mapper)
    }
}

//
// pub trait FnMapNonOptTrait<F> {
//     fn map(self, mapper: F) -> FnMapper<Self, F>
//     where
//         Self: Sized;
// }
//
// impl<S, F, U> FnMapNonOptTrait<F> for S
// where
//     S: NonOptParser,
//     F: Fn(S::Output) -> U,
// {
//     fn map(self, mapper: F) -> FnMapper<Self, F> {
//         FnMapper(self, mapper)
//     }
// }

//
// Or
//

pub struct OrPC<L, R>(L, R);

impl<L, R> HasOutput for OrPC<L, R>
where
    L: HasOutput,
    R: HasOutput<Output = L::Output>,
{
    type Output = L::Output;
}

impl<L, R> Parser for OrPC<L, R>
where
    L: Parser,
    R: Parser<Output = L::Output>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(first) => Ok(Some(first)),
            None => self.1.parse(tokenizer),
        }
    }
}

pub trait OrTrait<P> {
    fn or(self, other: P) -> OrPC<Self, P>
    where
        Self: Sized;
}

impl<S, P> OrTrait<P> for S
where
    S: Parser,
    P: Parser<Output = S::Output>,
{
    fn or(self, other: P) -> OrPC<Self, P> {
        OrPC(self, other)
    }
}

//
// AndOptFactory
//

pub struct AndOptFactoryPC<L, RF>(L, RF);

impl<L, RF, R> HasOutput for AndOptFactoryPC<L, RF>
where
    L: Parser,
    RF: Fn(&L::Output) -> R,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);
}

impl<L, RF, R> Parser for AndOptFactoryPC<L, RF>
where
    L: Parser,
    RF: Fn(&L::Output) -> R,
    R: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(first) => {
                let second_parser = (self.1)(&first);
                let opt_second = second_parser.parse(tokenizer)?;
                Ok(Some((first, opt_second)))
            }
            None => Ok(None),
        }
    }
}

pub trait AndOptFactoryTrait<F> {
    fn and_opt_factory(self, f: F) -> AndOptFactoryPC<Self, F>
    where
        Self: Sized;
}

impl<S, F, R> AndOptFactoryTrait<F> for S
where
    S: Parser,
    F: Fn(&S::Output) -> R,
    R: Parser,
{
    fn and_opt_factory(self, f: F) -> AndOptFactoryPC<Self, F> {
        AndOptFactoryPC(self, f)
    }
}

//
// Many
//

pub struct ManyParser<P> {
    parser: P,
    allow_empty: bool,
}

impl<P> HasOutput for ManyParser<P>
where
    P: HasOutput,
{
    type Output = Vec<P::Output>;
}

impl<P> Parser for ManyParser<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse(tokenizer)? {
                Some(value) => {
                    result.push(value);
                }
                None => {
                    break;
                }
            }
        }
        if result.is_empty() && !self.allow_empty {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

impl<P> NonOptParser for ManyParser<P>
where
    P: Parser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse(tokenizer)? {
                Some(value) => {
                    result.push(value);
                }
                None => {
                    break;
                }
            }
        }
        if result.is_empty() && !self.allow_empty {
            Err(QError::ArgumentCountMismatch)
        } else {
            Ok(result)
        }
    }
}

pub trait ManyTrait {
    fn zero_or_more(self) -> ManyParser<Self>
    where
        Self: Sized;
    fn one_or_more(self) -> ManyParser<Self>
    where
        Self: Sized;
}

impl<S> ManyTrait for S
where
    S: Parser,
{
    fn zero_or_more(self) -> ManyParser<Self> {
        ManyParser {
            parser: self,
            allow_empty: true,
        }
    }
    fn one_or_more(self) -> ManyParser<Self> {
        ManyParser {
            parser: self,
            allow_empty: false,
        }
    }
}
