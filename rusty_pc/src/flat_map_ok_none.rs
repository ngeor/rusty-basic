use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub struct FlatMapOkNoneParser<P, F, G> {
    parser: P,
    ok_mapper: F,
    incomplete_mapper: G,
}

impl<P, F, G> FlatMapOkNoneParser<P, F, G> {
    pub(crate) fn new(parser: P, ok_mapper: F, incomplete_mapper: G) -> Self {
        Self {
            parser,
            ok_mapper,
            incomplete_mapper,
        }
    }
}

impl<I, C, P, F, G, U> Parser<I, C> for FlatMapOkNoneParser<P, F, G>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(P::Output) -> Result<U, P::Error>,
    G: Fn() -> Result<U, P::Error>,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => (self.ok_mapper)(value),
            Err(err) if err.is_soft() => (self.incomplete_mapper)(),
            Err(err) => Err(err),
        }
    }
}

impl<C, P, F, G> SetContext<C> for FlatMapOkNoneParser<P, F, G>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
