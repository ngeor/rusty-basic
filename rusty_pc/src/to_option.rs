use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser};

pub struct ToOptionParser<P> {
    parser: P,
}

impl<P> ToOptionParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> MapDecorator<I, C> for ToOptionParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
{
    type OriginalOutput = P::Output;
    type Output = Option<P::Output>;
    type Error = P::Error;

    fn decorated(
        &mut self,
    ) -> &mut impl Parser<I, C, Output = Self::OriginalOutput, Error = Self::Error> {
        &mut self.parser
    }

    fn map_ok(&self, ok: Self::OriginalOutput) -> Self::Output {
        Some(ok)
    }

    fn map_soft_error(&self, _err: Self::Error) -> Result<Self::Output, Self::Error> {
        Ok(None)
    }
}

impl<P> MapDecoratorMarker for ToOptionParser<P> {}
