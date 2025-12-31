use crate::{ParseResult, ParseResultTrait, Parser};

pub trait FlatMap<I, C>: Parser<I, C>
where
    Self: Sized,
{
    /// Flat map the result of this parser for successful results.
    fn flat_map<F, U>(self, mapper: F) -> impl Parser<I, C, Output = U, Error = Self::Error>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, U, Self::Error>,
    {
        FlatMapParser(self, mapper)
    }
}

impl<I, C, P> FlatMap<I, C> for P where P: Parser<I, C> {}

struct FlatMapParser<P, F>(P, F);

impl<I, C, P, F, U> Parser<I, C> for FlatMapParser<P, F>
where
    P: Parser<I, C>,
    F: Fn(I, P::Output) -> ParseResult<I, U, P::Error>,
{
    type Output = U;
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.0.parse(tokenizer).flat_map(&self.1)
    }
}
