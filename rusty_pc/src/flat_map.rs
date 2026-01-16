use crate::{ParseResult, ParseResultTrait, Parser, SetContext};

pub trait FlatMap<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn flat_map<F, U>(self, mapper: F) -> FlatMapParser<Self, F>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, U, Self::Error>,
    {
        FlatMapParser::new(self, mapper)
    }
}
impl<I, C, P> FlatMap<I, C> for P where P: Parser<I, C> {}

pub struct FlatMapParser<P, F> {
    parser: P,
    mapper: F,
}
impl<P, F> FlatMapParser<P, F> {
    pub fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}
impl<I, C, P, F, U> Parser<I, C> for FlatMapParser<P, F>
where
    P: Parser<I, C>,
    F: Fn(I, P::Output) -> ParseResult<I, U, P::Error>,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&mut self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parser.parse(tokenizer).flat_map(&self.mapper)
    }
}
impl<C, P, F> SetContext<C> for FlatMapParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
