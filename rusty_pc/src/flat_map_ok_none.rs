use crate::{ParseResult, Parser, SetContext};

/// Flat map the result of this parser for successful and incomplete results.
/// Mapping is done by the given closures.
/// Other errors are never allowed to be re-mapped.
pub trait FlatMapOkNone<I, C>: Parser<I, C>
where
    Self: Sized,
{
    fn flat_map_ok_none<F, G, U>(
        self,
        ok_mapper: F,
        incomplete_mapper: G,
    ) -> FlatMapOkNoneParser<Self, F, G>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, U, Self::Error>,
        G: Fn(I) -> ParseResult<I, U, Self::Error>,
    {
        FlatMapOkNoneParser::new(self, ok_mapper, incomplete_mapper)
    }
}

impl<I, C, P> FlatMapOkNone<I, C> for P where P: Parser<I, C> {}

pub struct FlatMapOkNoneParser<P, F, G> {
    parser: P,
    ok_mapper: F,
    incomplete_mapper: G,
}

impl<P, F, G> FlatMapOkNoneParser<P, F, G> {
    pub fn new(parser: P, ok_mapper: F, incomplete_mapper: G) -> Self {
        Self {
            parser,
            ok_mapper,
            incomplete_mapper,
        }
    }
}

impl<I, C, P, F, G, U> Parser<I, C> for FlatMapOkNoneParser<P, F, G>
where
    P: Parser<I, C>,
    F: Fn(I, P::Output) -> ParseResult<I, U, P::Error>,
    G: Fn(I) -> ParseResult<I, U, P::Error>,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            Ok((input, value)) => (self.ok_mapper)(input, value),
            Err((false, i, _)) => (self.incomplete_mapper)(i),
            Err(err) => Err(err),
        }
    }
}

impl<C, P, F, G> SetContext<C> for FlatMapOkNoneParser<P, F, G>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait FlatMapNegateNone<I, C>: FlatMapOkNone<I, C>
where
    Self: Sized,
{
    /// Flat map the successful of this parser into an empty result.
    /// The Failed result is negated and mapped into an empty successful result (i.e. `None` becomes `Ok(())`).
    fn flat_map_negate_none<F>(
        self,
        ok_mapper: F,
    ) -> impl Parser<I, C, Output = (), Error = Self::Error>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, (), Self::Error>,
    {
        self.flat_map_ok_none(ok_mapper, |input| Ok((input, ())))
    }
}

impl<I, C, P> FlatMapNegateNone<I, C> for P where P: FlatMapOkNone<I, C> {}
