use crate::{InputTrait, Parser, SetContext};

pub struct FlatMapParser<P, F> {
    parser: P,
    mapper: F,
}
impl<P, F> FlatMapParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}
impl<I, C, P, F, U> Parser<I, C> for FlatMapParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(P::Output) -> Result<U, P::Error>,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&mut self, tokenizer: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.parse(tokenizer).and_then(&self.mapper)
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
