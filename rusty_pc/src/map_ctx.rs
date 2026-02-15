use crate::{InputTrait, Parser};

/// When setting the context of the underlying parser,
/// this parser will transform it using the given mapper.
pub struct MapCtxParser<P, F> {
    parser: P,
    context_mapper: F,
}

impl<P, F> MapCtxParser<P, F> {
    pub fn new<I, COut, CIn>(parser: P, context_mapper: F) -> Self
    where
        P: Parser<I, CIn>,
        F: Fn(&COut) -> CIn,
        I: InputTrait,
    {
        Self {
            parser,
            context_mapper,
        }
    }
}

impl<I, COut, CIn, P, F> Parser<I, COut> for MapCtxParser<P, F>
where
    I: InputTrait,
    P: Parser<I, CIn>,
    F: Fn(&COut) -> CIn,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.parse(input)
    }

    fn set_context(&mut self, ctx: &COut) {
        let ctx_in = (self.context_mapper)(ctx);
        self.parser.set_context(&ctx_in);
    }
}
