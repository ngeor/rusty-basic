use crate::{InputTrait, Parser, ParserErrorTrait, SetContext};

pub struct MapErrParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> MapErrParser<P, F> {
    pub(crate) fn new(parser: P, mapper: F) -> Self {
        Self { parser, mapper }
    }
}

impl<I, C, P, F> Parser<I, C> for MapErrParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    F: ErrorMapper<P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err(err) => Err(self.mapper.map(err)),
        }
    }
}

impl<C, P, E> SetContext<C> for MapErrParser<P, E>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait ErrorMapper<E>
where
    E: ParserErrorTrait,
{
    fn map(&self, err: E) -> E;
}

pub struct ToFatalErrorMapper;

impl<E> ErrorMapper<E> for ToFatalErrorMapper
where
    E: ParserErrorTrait,
{
    fn map(&self, err: E) -> E {
        err.to_fatal()
    }
}

pub struct SoftErrorOverrider<E>(E);

impl<E> SoftErrorOverrider<E> {
    pub fn new(err: E) -> Self {
        Self(err)
    }
}

impl<E> ErrorMapper<E> for SoftErrorOverrider<E>
where
    E: ParserErrorTrait,
{
    fn map(&self, err: E) -> E {
        if err.is_soft() { self.0.clone() } else { err }
    }
}

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
