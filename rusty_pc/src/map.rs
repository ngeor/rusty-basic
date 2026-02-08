use crate::{InputTrait, Parser, SetContext};

pub struct MapParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> MapParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, F, U> Parser<I, C> for MapParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(P::Output) -> U,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&mut self, tokenizer: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.parse(tokenizer).map(&self.mapper)
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

/// MapToUnitParser is the same as `.map(|_| ())`.
pub struct MapToUnitParser<P> {
    parser: P,
}

impl<P> MapToUnitParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> Parser<I, C> for MapToUnitParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
{
    type Output = ();
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

impl<C, P> SetContext<C> for MapToUnitParser<P>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
