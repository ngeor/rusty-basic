use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser, ParserErrorTrait};

pub struct ToFatalParser<P> {
    parser: P,
}

impl<P> ToFatalParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> MapDecorator<I, C> for ToFatalParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
{
    type OriginalOutput = P::Output;
    type Output = P::Output;
    type Error = P::Error;

    fn decorated(
        &mut self,
    ) -> &mut impl Parser<I, C, Output = Self::OriginalOutput, Error = Self::Error> {
        &mut self.parser
    }

    fn map_ok(&self, ok: Self::OriginalOutput) -> Result<Self::Output, Self::Error> {
        Ok(ok)
    }

    fn map_soft_error(&self, err: Self::Error) -> Result<Self::Output, Self::Error> {
        Err(err.to_fatal())
    }
}

impl<P> MapDecoratorMarker for ToFatalParser<P> {}
