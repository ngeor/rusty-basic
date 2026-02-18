use crate::{InputTrait, Parser, ParserErrorTrait};

/// A parser that maps the error of the decorated parser
/// using the given mapper.
pub struct MapErrParser<P, M> {
    parser: P,
    mapper: M,
}

impl<P, M> MapErrParser<P, M> {
    pub(crate) fn new(parser: P, mapper: M) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, M> Parser<I, C> for MapErrParser<P, M>
where
    I: InputTrait,
    P: Parser<I, C>,
    M: ErrorMapper<P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err(err) => Err(self.mapper.map(err)),
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.parser.set_context(ctx)
    }
}

/// Maps the error of the parser into a different error.
pub trait ErrorMapper<E>
where
    E: ParserErrorTrait,
{
    fn map(&self, err: E) -> E;
}

/// Overrides a fatal error with the given value.
pub struct FatalErrorOverrider<E>(E);

impl<E> FatalErrorOverrider<E> {
    pub fn new(err: E) -> Self {
        Self(err)
    }
}

impl<E> ErrorMapper<E> for FatalErrorOverrider<E>
where
    E: ParserErrorTrait,
{
    fn map(&self, err: E) -> E {
        if err.is_fatal() { self.0.clone() } else { err }
    }
}
