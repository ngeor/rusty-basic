use crate::{ParseResult, Parser, SetContext};

pub trait NoIncomplete<I, C>: Parser<I, C> + Sized {
    fn no_incomplete(self) -> MapErrParser<Self, Self::Error> {
        MapErrParser::new(self, true, None)
    }
}

impl<I, C, P> NoIncomplete<I, C> for P where P: Parser<I, C> + Sized {}

pub trait OrFail<I, C>: Parser<I, C> + Sized {
    fn or_fail(self, err: Self::Error) -> MapErrParser<Self, Self::Error> {
        MapErrParser::new(self, true, Some(err))
    }
}

impl<I, C, P> OrFail<I, C> for P where P: Parser<I, C> + Sized {}

pub struct MapErrParser<P, E> {
    parser: P,
    make_all_fatal: bool,
    override_non_fatal_error: Option<E>,
}

impl<P, E> MapErrParser<P, E> {
    pub fn new(parser: P, make_all_fatal: bool, override_non_fatal_error: Option<E>) -> Self {
        Self {
            parser,
            make_all_fatal,
            override_non_fatal_error,
        }
    }
}

impl<I, C, P, E> Parser<I, C> for MapErrParser<P, E>
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
                Err((self.make_all_fatal, i, err))
            }
            Err((true, i, err)) => Err((true, i, err)),
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
