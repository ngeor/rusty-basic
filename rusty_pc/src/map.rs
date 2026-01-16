use crate::{ParseResult, ParseResultTrait, Parser, SetContext};

pub trait Map<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn map<F, U>(self, mapper: F) -> MapParser<Self, F>
    where
        F: Fn(Self::Output) -> U,
    {
        MapParser::new(self, mapper)
    }
}
impl<I, C, P> Map<I, C> for P where P: Parser<I, C> {}

pub struct MapParser<P, F> {
    parser: P,
    mapper: F,
}
impl<P, F> MapParser<P, F> {
    pub fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}
impl<I, C, P, F, U> Parser<I, C> for MapParser<P, F>
where
    P: Parser<I, C>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&mut self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parser.parse(tokenizer).map_ok(&self.mapper)
    }
}
impl<C, P, F> SetContext<C> for MapParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
