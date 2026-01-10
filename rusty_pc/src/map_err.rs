use crate::{ParseResult, Parser, SetContext};

pub trait NoIncomplete<I, C>: Parser<I, C> + Sized {
    fn no_incomplete(self) -> ToFatalParser<Self, Self::Error> {
        ToFatalParser::new(self, None)
    }
}

impl<I, C, P> NoIncomplete<I, C> for P where P: Parser<I, C> + Sized {}

pub trait OrFail<I, C>: Parser<I, C> + Sized {
    fn or_fail(self, err: Self::Error) -> ToFatalParser<Self, Self::Error> {
        ToFatalParser::new(self, Some(err))
    }
}

impl<I, C, P> OrFail<I, C> for P where P: Parser<I, C> + Sized {}

pub struct ToFatalParser<P, E> {
    parser: P,
    override_non_fatal_error: Option<E>,
}

impl<P, E> ToFatalParser<P, E> {
    pub fn new(parser: P, override_non_fatal_error: Option<E>) -> Self {
        Self {
            parser,
            override_non_fatal_error,
        }
    }
}

impl<I, C, P, E> Parser<I, C> for ToFatalParser<P, E>
where
    P: Parser<I, C, Error = E>,
    E: Clone,
{
    type Output = P::Output;
    type Error = E;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err((false, i, err)) => {
                let err = match &self.override_non_fatal_error {
                    Some(e) => e.clone(),
                    _ => err,
                };
                Err((true, i, err))
            }
            Err(e) => Err(e),
        }
    }
}

impl<C, P, E> SetContext<C> for ToFatalParser<P, E>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait MapErr<I, C>: Parser<I, C> + Sized {
    fn map_fatal_err<F>(self, mapper: F) -> MapErrParser<Self, F> {
        MapErrParser::new(self, mapper, true, false)
    }
    fn map_non_fatal_err<F>(self, mapper: F) -> MapErrParser<Self, F> {
        MapErrParser::new(self, mapper, false, true)
    }
}

impl<I, C, P> MapErr<I, C> for P where P: Parser<I, C> + Sized {}

pub struct MapErrParser<P, F> {
    parser: P,
    mapper: F,
    map_fatal: bool,
    map_non_fatal: bool,
}

impl<P, F> MapErrParser<P, F> {
    pub fn new(parser: P, mapper: F, map_fatal: bool, map_non_fatal: bool) -> Self {
        Self {
            parser,
            mapper,
            map_fatal,
            map_non_fatal,
        }
    }
}

impl<I, C, P, F> Parser<I, C> for MapErrParser<P, F>
where
    P: Parser<I, C>,
    F: Fn(P::Error) -> P::Error,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err((false, i, err)) => Err((
                false,
                i,
                if self.map_non_fatal {
                    (self.mapper)(err)
                } else {
                    err
                },
            )),
            Err((true, i, err)) => Err((
                true,
                i,
                if self.map_fatal {
                    (self.mapper)(err)
                } else {
                    err
                },
            )),
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
