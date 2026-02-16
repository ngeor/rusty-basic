use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser};

pub struct MapParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> MapParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, F, U> MapDecorator<I, C> for MapParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(P::Output) -> U,
{
    type OriginalOutput = P::Output;
    type Output = U;
    type Error = P::Error;

    fn decorated(&mut self) -> &mut impl Parser<I, C, Output = P::Output, Error = P::Error> {
        &mut self.parser
    }

    fn map_ok(&self, ok: P::Output) -> U {
        (self.mapper)(ok)
    }
}

impl<P, F> MapDecoratorMarker for MapParser<P, F> {}

/// MapToUnitParser is the same as `.map(|_| ())`.
pub struct MapToUnitParser<P> {
    parser: P,
}

impl<P> MapToUnitParser<P> {
    pub(crate) fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> MapDecorator<I, C> for MapToUnitParser<P>
where
    I: InputTrait,
    P: Parser<I, C>,
{
    type OriginalOutput = P::Output;
    type Output = ();
    type Error = P::Error;

    fn decorated(&mut self) -> &mut impl Parser<I, C, Output = P::Output, Error = P::Error> {
        &mut self.parser
    }

    fn map_ok(&self, _ok: P::Output) {}
}

impl<P> MapDecoratorMarker for MapToUnitParser<P> {}
