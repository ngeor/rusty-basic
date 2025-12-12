//! Mappers that are able to return an error

use crate::pc::*;
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct AndThen<mapper: F>);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for AndThen<P, F>
where
    P: Parser<I>,
    F: Fn(P::Output) -> Result<U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).and_then(&self.mapper)
    }
}

parser_declaration!(pub struct AndThenOkErr<ok_mapper: F, err_mapper: G>);

impl<I: Tokenizer + 'static, P, F, G, U> Parser<I> for AndThenOkErr<P, F, G>
where
    P: Parser<I>,
    F: Fn(P::Output) -> Result<U, ParseError>,
    G: Fn(ParseError) -> Result<U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => (self.ok_mapper)(value),
            Err(err) => (self.err_mapper)(err),
        }
    }
}
