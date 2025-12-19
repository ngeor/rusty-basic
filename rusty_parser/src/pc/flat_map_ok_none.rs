use crate::pc::*;
use crate::ParseError;

pub trait FlatMapOkNone<I: Tokenizer + 'static>: Parser<I> {
    /// Flat map the result of this parser for successful and incomplete results.
    /// Mapping is done by the given closures.
    /// Other errors are never allowed to be re-mapped.
    fn flat_map_ok_none<F, G, U>(
        self,
        ok_mapper: F,
        incomplete_mapper: G,
    ) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>,
        G: Fn() -> ParseResult<U, ParseError>;

    /// Flat map the successful of this parser into an empty result.
    /// The Failed result is negated and mapped into an empty successful result (i.e. `None` becomes `Ok(())`).
    fn flat_map_negate_none<F>(self, ok_mapper: F) -> impl Parser<I, Output = ()>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<(), ParseError>,
    {
        self.flat_map_ok_none(ok_mapper, || ParseResult::Ok(()))
    }
}

impl<I, P> FlatMapOkNone<I> for P
where
    I: Tokenizer + 'static,
    P: Parser<I>,
{
    fn flat_map_ok_none<F, G, U>(
        self,
        ok_mapper: F,
        incomplete_mapper: G,
    ) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        F: Fn(Self::Output) -> ParseResult<U, ParseError>,
        G: Fn() -> ParseResult<U, ParseError>,
    {
        FlatMapOkNoneParser::new(self, ok_mapper, incomplete_mapper)
    }
}

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

impl<I: Tokenizer + 'static, P, F, G, U> Parser<I> for FlatMapOkNoneParser<P, F, G>
where
    P: Parser<I>,
    F: Fn(P::Output) -> ParseResult<U, ParseError>,
    G: Fn() -> ParseResult<U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => (self.ok_mapper)(value),
            ParseResult::None | ParseResult::Expected(_) => (self.incomplete_mapper)(),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
