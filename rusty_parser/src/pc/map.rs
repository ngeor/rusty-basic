use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::ParseError;

pub trait Map<I: Tokenizer + 'static>: Parser<I> {
    fn map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U;

    #[deprecated]
    fn keep_right<L, R>(self) -> impl Parser<I, Output = R>
    where
        Self: Sized + Parser<I, Output = (L, R)>,
    {
        self.map(|(_, r)| r)
    }
}

impl<I, P> Map<I> for P
where
    I: Tokenizer + 'static,
    P: Parser<I>,
{
    fn map<F, U>(self, mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        MapParser(self, mapper)
    }
}

struct MapParser<P, F>(P, F);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for MapParser<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.0.parse(tokenizer).map(&self.1)
    }
}
