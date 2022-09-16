use super::tokenizers::{Token, Tokenizer};
use crate::common::QError;

// Parser

pub trait Parser {
    type Item;
    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError>;
}

// FilterTokenByKindParser

struct FilterTokenByKindParser<'a> {
    kind: i32,
    optional: bool,
    err_msg: &'a str,
}

impl<'a> Parser for FilterTokenByKindParser<'a> {
    type Item = Token;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        match source.read() {
            Ok(Some(token)) => {
                if token.kind == self.kind {
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

pub fn filter_token_by_kind_opt(kind: i32) -> impl Parser<Item = Token> {
    FilterTokenByKindParser {
        kind,
        optional: true,
        err_msg: "",
    }
}

pub fn filter_token_by_kind<'a>(kind: i32, err_msg: &'a str) -> impl Parser<Item = Token> + 'a {
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
    type Item = Token;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
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

pub fn filter_token<P: Fn(&Token) -> bool>(predicate: P) -> impl Parser<Item = Token> {
    FilterTokenParser { predicate }
}

// Map

struct MapParser<P, M, U>
where
    P: Parser,
    M: Fn(P::Item) -> U,
{
    parser: P,
    mapper: M,
}

impl<P, M, U> Parser for MapParser<P, M, U>
where
    P: Parser,
    M: Fn(P::Item) -> U,
{
    type Item = U;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        match self.parser.parse(source) {
            Ok(Some(x)) => Ok(Some((self.mapper)(x))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub fn map<P, M, U>(parser: P, mapper: M) -> impl Parser<Item = U>
where
    P: Parser,
    M: Fn(P::Item) -> U,
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
    type Item = (L::Item, R::Item);

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
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

pub fn and<L, R>(left: L, right: R) -> impl Parser<Item = (L::Item, R::Item)>
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
) -> impl Parser<Item = (P1::Item, P2::Item, P3::Item)>
where
    P1: Parser,
    P2: Parser,
    P3: Parser,
{
    map(and(p1, and(p2, p3)), |(a, (b, c))| (a, b, c))
}
