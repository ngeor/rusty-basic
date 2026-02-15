use std::marker::PhantomData;

use crate::{InputTrait, Parser};

/// Stops propagating the context to the underlying parser.
/// The underlying parser might also have a different context type,
/// so this can be used to resolve context type mismatches,
/// as long as the underlying parser does not use the parent context.
pub struct NoContextParser<P, COut, CIn> {
    parser: P,
    _marker: PhantomData<(COut, CIn)>,
}

impl<P, COut, CIn> NoContextParser<P, COut, CIn> {
    pub(crate) fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}

impl<I, COut, CIn, P> Parser<I, COut> for NoContextParser<P, COut, CIn>
where
    P: Parser<I, CIn>,
    I: InputTrait,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.parse(input)
    }

    fn set_context(&mut self, _ctx: &COut) {}
}
