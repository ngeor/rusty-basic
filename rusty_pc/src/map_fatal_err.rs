use crate::{InputTrait, Parser, ParserErrorTrait};

/// A parser that maps the error of the decorated parser
/// using the given mapper.
pub struct MapFatalErrParser<P, E> {
    parser: P,
    err: E,
}

impl<P, E> MapFatalErrParser<P, E> {
    pub(crate) fn new(parser: P, err: E) -> Self {
        Self { parser, err }
    }
}

impl<I, C, P, E> Parser<I, C> for MapFatalErrParser<P, E>
where
    I: InputTrait,
    P: Parser<I, C, Error = E>,
    E: ParserErrorTrait,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(value) => Ok(value),
            Err(_) => Err(self.err.clone()),
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.parser.set_context(ctx)
    }
}
