use crate::{ParseResult, Parser};

pub trait NoIncomplete<I>: Parser<I> + Sized {
    fn no_incomplete(self) -> MapErrParser<Self, Self::Error> {
        MapErrParser::new(self, true, None)
    }
}

impl<I, P> NoIncomplete<I> for P where P: Parser<I> + Sized {}

pub trait OrFail<I>: Parser<I> + Sized {
    fn or_fail(self, err: Self::Error) -> MapErrParser<Self, Self::Error> {
        MapErrParser::new(self, true, Some(err))
    }
}

impl<I, P> OrFail<I> for P where P: Parser<I> + Sized {}

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

impl<I, P, E> Parser<I> for MapErrParser<P, E>
where
    P: Parser<I, Error = E>,
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
