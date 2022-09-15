use super::tokenizers::{Token, Tokenizer};
use crate::common::QError;

pub trait Parser {
    type Item;
    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError>;
}

struct AnyTokenParser {}

impl Parser for AnyTokenParser {
    type Item = Token;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        source.read().map_err(|e| e.into())
    }
}

pub fn any_token() -> impl Parser<Item = Token> {
    AnyTokenParser {}
}

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

struct TokenAndTokenParser<L: Parser<Item = Token>, R: Parser<Item = Token>> {
    left: L,
    right: R,
}

impl<L: Parser<Item = Token>, R: Parser<Item = Token>> Parser for TokenAndTokenParser<L, R> {
    type Item = (L::Item, R::Item);

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        match self.left.parse(source) {
            Ok(Some(left)) => match self.right.parse(source) {
                Ok(Some(right)) => Ok(Some((left, right))),
                Ok(None) => {
                    source.unread(left);
                    Ok(None)
                }
                Err(err) => Err(err),
            },
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub fn seq<L: Parser<Item = Token>, R: Parser<Item = Token>>(
    left: L,
    right: R,
) -> impl Parser<Item = (L::Item, R::Item)> {
    TokenAndTokenParser { left, right }
}

struct TokenPairAndTokenParser<L: Parser<Item = (Token, Token)>, R: Parser<Item = Token>> {
    left: L,
    right: R,
}

impl<L: Parser<Item = (Token, Token)>, R: Parser<Item = Token>> Parser
    for TokenPairAndTokenParser<L, R>
{
    type Item = (Token, Token, Token);

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        match self.left.parse(source) {
            Ok(Some((left1, left2))) => match self.right.parse(source) {
                Ok(Some(right)) => Ok(Some((left1, left2, right))),
                Ok(None) => {
                    source.unread(left2);
                    source.unread(left1);
                    Ok(None)
                }
                Err(err) => Err(err),
            },
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub fn seq3<P1: Parser<Item = Token>, P2: Parser<Item = Token>, P3: Parser<Item = Token>>(
    p1: P1,
    p2: P2,
    p3: P3,
) -> impl Parser<Item = (Token, Token, Token)> {
    TokenPairAndTokenParser {
        left: seq(p1, p2),
        right: p3,
    }
}

struct DemandParser<'a, P>
where
    P: Parser,
{
    parser: P,
    err: &'a str,
}

impl<'a, P> Parser for DemandParser<'a, P>
where
    P: Parser,
{
    type Item = P::Item;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Item>, QError> {
        match self.parser.parse(source) {
            Ok(Some(x)) => Ok(Some(x)),
            Ok(None) => Err(QError::SyntaxError(String::from(self.err))),
            Err(err) => Err(err),
        }
    }
}

pub fn demand<'a, P: 'a + Parser>(parser: P, err: &'a str) -> impl Parser<Item = P::Item> + 'a {
    DemandParser { parser, err }
}

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
