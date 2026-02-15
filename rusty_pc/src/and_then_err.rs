use crate::{InputTrait, Parser, ParserErrorTrait};

pub struct AndThenErrParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> AndThenErrParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, F> Parser<I, C> for AndThenErrParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(P::Error) -> Result<P::Output, P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err(err) if err.is_soft() => (self.mapper)(err),
            Err(err) => Err(err),
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.parser.set_context(ctx)
    }
}
