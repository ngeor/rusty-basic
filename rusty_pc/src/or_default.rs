use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser};

pub struct OrDefaultParser<P> {
    parser: P,
}

impl<P> OrDefaultParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> MapDecorator<I, C> for OrDefaultParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
    P::Output: Default,
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

    fn map_soft_error(&self, _err: Self::Error) -> Result<Self::Output, Self::Error> {
        Ok(P::Output::default())
    }
}

impl<P> MapDecoratorMarker for OrDefaultParser<P> {}
