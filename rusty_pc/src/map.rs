use crate::{ParseResult, ParseResultTrait, Parser};

pub trait Map<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn map<F, U>(self, mapper: F) -> impl Parser<I, C, Output = U, Error = Self::Error>
    where
        F: Fn(Self::Output) -> U,
    {
        MapParser(self, mapper)
    }
}

impl<I, C, P> Map<I, C> for P where P: Parser<I, C> {}

struct MapParser<P, F>(P, F);

impl<I, C, P, F, U> Parser<I, C> for MapParser<P, F>
where
    P: Parser<I, C>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.0.parse(tokenizer).map_ok(&self.1)
    }
}
