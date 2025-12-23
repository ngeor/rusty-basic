use crate::error::ParseError;
use crate::pc::*;

pub trait FlatMapOkNone<I>: Parser<I>
where
    Self: Sized,
{
    /// Flat map the result of this parser for successful and incomplete results.
    /// Mapping is done by the given closures.
    /// Other errors are never allowed to be re-mapped.
    fn flat_map_ok_none<F, G, U>(
        self,
        ok_mapper: F,
        incomplete_mapper: G,
    ) -> impl Parser<I, Output = U>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, U, ParseError>,
        G: Fn(I) -> ParseResult<I, U, ParseError>,
    {
        FlatMapOkNoneParser::new(self, ok_mapper, incomplete_mapper)
    }

    /// Flat map the successful of this parser into an empty result.
    /// The Failed result is negated and mapped into an empty successful result (i.e. `None` becomes `Ok(())`).
    fn flat_map_negate_none<F>(self, ok_mapper: F) -> impl Parser<I, Output = ()>
    where
        F: Fn(I, Self::Output) -> ParseResult<I, (), ParseError>,
    {
        self.flat_map_ok_none(ok_mapper, |input| Ok((input, ())))
    }
}

impl<I, P> FlatMapOkNone<I> for P where P: Parser<I> {}

struct FlatMapOkNoneParser<P, F, G> {
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

impl<I, P, F, G, U> Parser<I> for FlatMapOkNoneParser<P, F, G>
where
    P: Parser<I>,
    F: Fn(I, P::Output) -> ParseResult<I, U, ParseError>,
    G: Fn(I) -> ParseResult<I, U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok((input, value)) => (self.ok_mapper)(input, value),
            Err((false, i, _)) => (self.incomplete_mapper)(i),
            Err(err) => Err(err),
        }
    }
}
