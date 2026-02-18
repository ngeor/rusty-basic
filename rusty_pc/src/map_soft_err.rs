use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser, ParserErrorTrait};

pub struct MapSoftErrParser<P, E> {
    parser: P,
    err: E,
}

impl<P, E> MapSoftErrParser<P, E> {
    pub(crate) fn new(parser: P, err: E) -> Self {
        Self { parser, err }
    }
}

impl<I, C, P, E> MapDecorator<I, C> for MapSoftErrParser<P, E>
where
    I: InputTrait,
    P: Parser<I, C, Error = E>,
    E: ParserErrorTrait,
{
    type OriginalOutput = P::Output;
    type Output = P::Output;
    type Error = E;

    fn decorated(
        &mut self,
    ) -> &mut impl Parser<I, C, Output = Self::OriginalOutput, Error = Self::Error> {
        &mut self.parser
    }

    fn map_ok(&self, ok: Self::OriginalOutput) -> Result<Self::Output, Self::Error> {
        Ok(ok)
    }

    fn map_soft_error(&self, _err: Self::Error) -> Result<Self::Output, Self::Error> {
        Err(self.err.clone())
    }
}

impl<P, E> MapDecoratorMarker for MapSoftErrParser<P, E> {}
