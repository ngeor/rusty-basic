use crate::{ParseResult, Parser, ParserErrorTrait, SetContext};

/// Flat map the result of this parser for successful and incomplete results.
/// Mapping is done by the given closures.
/// Other errors are never allowed to be re-mapped.
pub trait FlatMapNone<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn flat_map_none<F>(self, incomplete_mapper: F) -> FlatMapNoneParser<Self, F>
    where
        F: Fn(I) -> ParseResult<I, Self::Output, Self::Error>,
    {
        FlatMapNoneParser::new(self, incomplete_mapper)
    }
}

impl<I, C, P> FlatMapNone<I, C> for P where P: Parser<I, C> {}

pub struct FlatMapNoneParser<P, F> {
    parser: P,
    incomplete_mapper: F,
}

impl<P, F> FlatMapNoneParser<P, F> {
    pub fn new(parser: P, incomplete_mapper: F) -> Self {
        Self {
            parser,
            incomplete_mapper,
        }
    }
}

impl<I, C, P, F> Parser<I, C> for FlatMapNoneParser<P, F>
where
    P: Parser<I, C>,
    F: Fn(I) -> ParseResult<I, P::Output, P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok(o) => Ok(o),
            Err((i, err)) if !err.is_fatal() => (self.incomplete_mapper)(i),
            Err(err) => Err(err),
        }
    }
}

impl<C, P, F> SetContext<C> for FlatMapNoneParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
