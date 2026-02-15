use crate::{InputTrait, Parser};

pub struct AndThenParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> AndThenParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, F, U> Parser<I, C> for AndThenParser<P, F>
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

    fn set_context(&mut self, ctx: &C) {
        self.parser.set_context(ctx)
    }
}
