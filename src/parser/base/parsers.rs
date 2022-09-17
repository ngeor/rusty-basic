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

pub fn filter_token_by_kind_opt<T: Copy>(kind: T) -> impl Parser<Output = Token> {
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
    P: Fn(&Token) -> Result<bool, QError>,
{
    predicate: P,
}

impl<P> Parser for FilterTokenParser<P>
where
    P: Fn(&Token) -> Result<bool, QError>,
{
    type Output = Token;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match source.read() {
            Ok(Some(token)) => {
                if (self.predicate)(&token)? {
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

pub fn filter_token<P: Fn(&Token) -> Result<bool, QError>>(
    predicate: P,
) -> impl Parser<Output = Token> {
    FilterTokenParser { predicate }
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

// Or Parser

struct OrParser<L, R, T>
where
    L: Parser<Output = T>,
    R: Parser<Output = T>,
{
    left: L,
    right: R,
}

impl<L, R, T> Parser<Output = T> for OrParser<L, R, T>
where
    L: Parser<Output = T>,
    R: Parser<Output = T>,
{
    type Output = ();

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.left.parse(tokenizer)? {
            Some(a) => Ok(Some(a)),
            None => self.right.parse(tokenizer),
        }
    }
}

pub fn alt<L, R, T>(left: L, right: L) -> impl Parser<Output = T>
where
    L: Parser<Output = T>,
    R: Parser<Output = T>,
{
    OrParser { left, right }
}

pub fn alt3<A, B, C, T>(a: A, b: B, c: C) -> impl Parser<Output = T>
where
    A: Parser<Output = T>,
    B: Parser<Output = T>,
    C: Parser<Output = T>,
{
    alt(a, alt(b, c))
}

pub fn alt4<A, B, C, D, T>(a: A, b: B, c: C, d: D) -> impl Parser<Output = T>
where
    A: Parser<Output = T>,
    B: Parser<Output = T>,
    C: Parser<Output = T>,
    D: Parser<Output = T>,
{
    alt(a, alt3(b, c, d))
}

pub fn alt5<A, B, C, D, E, T>(a: A, b: B, c: C, d: D, e: E) -> impl Parser<Output = T>
where
    A: Parser<Output = T>,
    B: Parser<Output = T>,
    C: Parser<Output = T>,
    D: Parser<Output = T>,
    E: Parser<Output = T>,
{
    alt(a, alt4(b, c, d, e))
}

pub fn alt6<A, B, C, D, E, F, T>(a: A, b: B, c: C, d: D, e: E, f: F) -> impl Parser<Output = T>
where
    A: Parser<Output = T>,
    B: Parser<Output = T>,
    C: Parser<Output = T>,
    D: Parser<Output = T>,
    E: Parser<Output = T>,
    F: Parser<Output = T>,
{
    alt(a, alt5(b, c, d, e, f))
}

// And Then

struct AndThenParser<P, M, U>
where
    P: Parser,
    M: Fn(P::Output) -> Result<Option<U>, QError>,
{
    parser: P,
    mapper: M,
}

impl<P, M, U> Parser for AndThenParser<P, M, U>
where
    P: Parser,
    M: Fn(P::Output) -> Result<Option<U>, QError>,
{
    type Output = U;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(source) {
            Ok(Some(x)) => (self.mapper)(x),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub fn and_then<P, M, U>(parser: P, mapper: M) -> impl Parser<Output = U>
where
    P: Parser,
    M: Fn(P::Output) -> Result<Option<U>, QError>,
{
    AndThenParser { parser, mapper }
}

pub fn map<P, M, U>(parser: P, mapper: M) -> impl Parser<Output = U>
where
    P: Parser,
    M: Fn(P::Output) -> U,
{
    and_then(parser, |x| Ok(Some(mapper(x))))
}
