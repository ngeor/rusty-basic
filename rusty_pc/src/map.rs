use crate::{ParseResult, ParseResultTrait, Parser};

pub trait Map<I>: Parser<I>
where
    Self: Sized,
{
    fn map<F, U>(self, mapper: F) -> impl Parser<I, Output = U, Error = Self::Error>
    where
        F: Fn(Self::Output) -> U,
    {
        MapParser(self, mapper)
    }
}

impl<I, P> Map<I> for P where P: Parser<I> {}

struct MapParser<P, F>(P, F);

impl<I, P, F, U> Parser<I> for MapParser<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.0.parse(tokenizer).map_ok(&self.1)
    }
}
