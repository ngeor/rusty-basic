use crate::{ParseResult, Parser};

pub(crate) struct MapErrParser<P, E> {
    parser: P,
    make_all_fatal: bool,
    override_non_fatal_error: Option<E>,
}

impl<P, E> MapErrParser<P, E> {
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            make_all_fatal: false,
            override_non_fatal_error: None,
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

    fn no_incomplete(self) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
    {
        Self {
            make_all_fatal: true,
            ..self
        }
    }

    fn or_fail(self, err: Self::Error) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        Self::Error: Clone,
    {
        Self {
            make_all_fatal: true,
            override_non_fatal_error: Some(err),
            ..self
        }
    }
}
