use std::marker::PhantomData;

use crate::{InputTrait, Parser, SetContext};

pub struct FlattenParser<P, CIn> {
    parser: P,
    _marker: PhantomData<CIn>,
}

impl<P, CIn> FlattenParser<P, CIn> {
    pub(crate) fn new(parser: P) -> Self {
        Self {
            parser,
            _marker: PhantomData,
        }
    }
}

impl<I, COut, CIn, P> Parser<I, COut> for FlattenParser<P, CIn>
where
    I: InputTrait,
    P: Parser<I, COut>,
    P::Output: Parser<I, CIn, Error = P::Error>,
{
    type Output = <P::Output as Parser<I, CIn>>::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(mut new_parser) => new_parser.parse(input),
            Err(err) => Err(err),
        }
    }
}
impl<P, COut, CIn> SetContext<COut> for FlattenParser<P, CIn>
where
    P: SetContext<COut>,
{
    fn set_context(&mut self, ctx: COut) {
        self.parser.set_context(ctx)
    }
}
