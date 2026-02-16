use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser};

pub struct AndThenErrParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> AndThenErrParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, F> MapDecorator<I, C> for AndThenErrParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(P::Error) -> Result<P::Output, P::Error>,
{
    type OriginalOutput = P::Output;
    type Output = P::Output;
    type Error = P::Error;

    fn decorated(
        &mut self,
    ) -> &mut impl Parser<I, C, Output = Self::OriginalOutput, Error = Self::Error> {
        &mut self.parser
    }

    fn map_ok(&self, ok: Self::OriginalOutput) -> Self::Output {
        ok
    }

    fn map_soft_error(&self, err: Self::Error) -> Result<Self::Output, Self::Error> {
        (self.mapper)(err)
    }
}

impl<P, F> MapDecoratorMarker for AndThenErrParser<P, F> {}
