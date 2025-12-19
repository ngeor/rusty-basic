use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::ParseError;

pub trait FlatMap<I: Tokenizer + 'static>: Parser<I> {
    /// Flat map the result of this parser for successful results.
    fn flat_map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>;
}

impl<I, P> FlatMap<I> for P
where
    I: Tokenizer + 'static,
    P: Parser<I>,
{
    fn flat_map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>,
    {
        FlatMapParser(self, mapper)
    }
}

struct FlatMapParser<P, F>(P, F);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for FlatMapParser<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> ParseResult<U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.0.parse(tokenizer).flat_map(&self.1)
    }
}
