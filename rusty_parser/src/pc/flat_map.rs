use crate::pc::*;
use crate::{parser_declaration, ParseError};

// Flat map the successful result.

parser_declaration!(pub struct FlatMapPC<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for FlatMapPC<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> ParseResult<U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.parser.parse(tokenizer).flat_map(&self.mapper)
    }
}

// Flat map Ok and None using closures.

parser_declaration!(pub struct FlatMapOkNoneClosuresPC<ok_mapper: F, incomplete_mapper: G>);

impl<I: Tokenizer + 'static, P, F, G, U> Parser<I> for FlatMapOkNoneClosuresPC<P, F, G>
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
