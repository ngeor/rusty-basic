use super::tokenizers::{Token, Tokenizer};
use crate::common::QError;

// Parser

pub trait Parser {
    type Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError>;
}

// FilterTokenByKindParser

struct FilterTokenByKindParser<'a, T>
where
    T: Copy,
{
    kind: T,
    optional: bool,
    err_msg: &'a str,
}

impl<'a, T> Parser for FilterTokenByKindParser<'a, T>
where
    T: Copy,
{
    type Output = Token;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match source.read() {
            Ok(Some(token)) => {
                if token.kind == self.kind as i32 {
                    Ok(Some(token))
                } else {
                    if self.optional {
                        source.unread(token);
                        Ok(None)
                    } else {
                        Err(QError::SyntaxError(self.err_msg.into()))
                    }
                }
            }
            Ok(None) => {
                if self.optional {
                    Ok(None)
                } else {
                    Err(QError::SyntaxError(self.err_msg.into()))
                }
            }
            Err(err) => Err(err.into()),
        }
    }
}

pub fn filter_token_by_kind_opt<T: Copy>(kind: T) -> impl Parser<Item = Token> {
    FilterTokenByKindParser {
        kind,
        optional: true,
        err_msg: "",
    }
}

pub fn filter_token_by_kind<'a, T: Copy>(
    kind: T,
    err_msg: &'a str,
) -> impl Parser<Item = Token> + 'a {
    FilterTokenByKindParser {
        kind,
        optional: false,
        err_msg,
    }
}

// FilterTokenParser

struct FilterTokenParser<P>
where
    P: Fn(&Token) -> bool,
{
    predicate: P,
}

impl<P> Parser for FilterTokenParser<P>
where
    P: Fn(&Token) -> bool,
{
    type Output = Token;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match source.read() {
            Ok(Some(token)) => {
                if (self.predicate)(&token) {
                    Ok(Some(token))
                } else {
                    source.unread(token);
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

pub fn filter_token<P: Fn(&Token) -> bool>(predicate: P) -> impl Parser<Output = Token> {
    FilterTokenParser { predicate }
}

// Map

struct MapParser<P, M, U>
where
    P: Parser,
    M: Fn(P::Output) -> U,
{
    parser: P,
    mapper: M,
}

impl<P, M, U> Parser for MapParser<P, M, U>
where
    P: Parser,
    M: Fn(P::Output) -> U,
{
    type Output = U;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(source) {
            Ok(Some(x)) => Ok(Some((self.mapper)(x))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub fn map<P, M, U>(parser: P, mapper: M) -> impl Parser<Output = U>
where
    P: Parser,
    M: Fn(P::Output) -> U,
{
    MapParser { parser, mapper }
}

// And (no undo!)

struct AndParser<L, R>
where
    L: Parser,
    R: Parser,
{
    left: L,
    right: R,
}

impl<L, R> Parser for AndParser<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = (L::Output, R::Output);

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.left.parse(source)? {
            Some(first) => {
                match self.right.parse(source)? {
                    Some(second) => Ok(Some((first, second))),
                    None => {
                        // should not happen!
                        Err(QError::InternalError(
                            "AndParser does not support undo! The right parser returned None"
                                .into(),
                        ))
                    }
                }
            }
            None => Ok(None),
        }
    }
}

pub fn and<L, R>(left: L, right: R) -> impl Parser<Output = (L::Output, R::Output)>
where
    L: Parser,
    R: Parser,
{
    AndParser { left, right }
}

pub fn seq3<P1, P2, P3>(
    p1: P1,
    p2: P2,
    p3: P3,
) -> impl Parser<Output = (P1::Output, P2::Output, P3::Output)>
where
    P1: Parser,
    P2: Parser,
    P3: Parser,
{
    map(and(p1, and(p2, p3)), |(a, (b, c))| (a, b, c))
}

// Many

struct Many<P> {
    parser: P,
    allow_empty: bool,
}

impl<P: Parser> Parser for Many<P> {
    type Output = Vec<P::Output>;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse(source)? {
                Some(item) => {
                    result.push(item);
                }
                None => {
                    break;
                }
            }
        }
        if self.allow_empty || !result.is_empty() {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

pub fn many_opt<P: Parser>(parser: P) -> impl Parser<Output = Vec<P::Output>> {
    Many {
        parser,
        allow_empty: true,
    }
}
