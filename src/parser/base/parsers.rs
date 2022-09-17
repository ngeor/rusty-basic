use super::tokenizers::{Token, Tokenizer};
use crate::common::QError;

// Parser

pub trait Parser {
    type Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError>;

    fn or<P>(self, other: P) -> OrParser<Self, P, Self::Output>
    where
        Self: Sized,
        P: Parser<Output = Self::Output>,
    {
        OrParser {
            left: self,
            right: other,
        }
    }

    fn and_opt<P>(self, other: P) -> AndOptParser<Self, P>
    where
        Self: Sized,
        P: Parser,
    {
        AndOptParser {
            left: self,
            right: other,
        }
    }

    fn and<P>(self, other: P) -> AndParser<Self, P>
    where
        Self: Sized,
        P: Parser,
    {
        AndParser {
            left: self,
            right: other,
        }
    }

    fn surrounded_by_opt<F>(self, predicate: F) -> SurroundedByOptParser<Self, F>
    where
        Self: Sized,
        F: Fn(&Token, bool),
    {
        SurroundedByOptParser {
            parser: self,
            predicate,
        }
    }

    fn one_or_more(self) -> ManyParser<Self>
    where
        Self: Sized,
    {
        ManyParser {
            parser: self,
            allow_empty: false,
        }
    }

    fn map<F, U>(self, mapper: F) -> MapParser<Self, U> where Self : Sized, F : Fn(Self::Output) -> U {
        map(self, mapper)
    }
}

// TODO make a new trait for a Parser that is guaranteed to not return Ok(None)

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
) -> impl Parser<Output = Token> + 'a {
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

pub struct AndParser<L, R>
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

impl<L, R> AndParser<L, R>
where
    L: Parser,
    R: Parser,
{
    pub fn keep_right(self) -> impl Parser<Output = R> {
        keep_right(self)
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

pub struct ManyParser<P> {
    parser: P,
    allow_empty: bool,
}

impl<P: Parser> Parser for ManyParser<P> {
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
    ManyParser {
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

impl<L, R, T> Parser for OrParser<L, R, T>
where
    L: Parser<Output = T>,
    R: Parser<Output = T>,
{
    type Output = T;

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

// Map Parser

pub struct MapParser<P, M, U>
    where
        P: Parser,
        M: Fn(P::Output) -> U,
{
    parser: P,
    mapper: M,
}

impl<P, M, U> Parser for AndThenParser<P, M, U>
    where
        P: Parser,
        M: Fn(P::Output) -> U
{
    type Output = U;

    fn parse(&self, source: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser.parse(source).map(|opt_x| opt_x.map(|x| (self.mapper)(x)))
    }
}


pub fn map<P, M, U>(parser: P, mapper: M) -> impl Parser<Output = U>
where
    P: Parser,
    M: Fn(P::Output) -> U,
{
    MapParser { parser, mapper }
}

// And Opt

pub struct AndOptParser<L, R> {
    left: L,
    right: R,
}

impl<L, R> Parser for AndOptParser<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.left.parse(tokenizer)? {
            Some(a) => {
                let opt_b = self.right.parse(tokenizer)?;
                Ok(Some((a, opt_b)))
            }
            None => Ok(None),
        }
    }
}

impl<L, R> AndOptParser<L, R>
where
    L: Parser,
    R: Parser,
{
    pub fn keep_right(self) -> impl Parser<Output = Option<R>> {
        keep_right(self)
    }
}

pub fn keep_right<P, L, R>(parser: P) -> impl Parser<Output = R>
where
    P: Parser<Output = (L, R)>,
{
    map(parser, |(l, r)| r)
}

// Surrounded By Opt

pub struct SurroundedByOptParser<P, F> {
    parser: P,
    predicate: F,
}

impl<P, F> Parser for SurroundedByOptParser<P, F>
where
    P: Parser,
    F: Fn(&Token) -> bool,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let leading_token = self.read_surrounding_token(tokenizer)?;
        match self.parser.parse(tokenizer)? {
            Some(result) => {
                // ok read the trailing token too if it exists
                self.read_surrounding_token(tokenizer)?;
                Ok(Some(result))
            }
            None => {
                // undo the leading token
                if let Some(token) = leading_token {
                    tokenizer.unread(token);
                }
                Ok(None)
            }
        }
    }
}

impl<P, F> SurroundedByOptParser<P, F>
where
    P: Parser,
    F: Fn(&Token) -> bool,
{
    fn read_surrounding_token(
        &self,
        tokenizer: &mut impl Tokenizer,
    ) -> Result<Option<Token>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if (self.predicate)(&token) {
                    // keep it for now
                    Ok(Some(token))
                } else {
                    // undo but it is okay to continue
                    tokenizer.unread(token);
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
