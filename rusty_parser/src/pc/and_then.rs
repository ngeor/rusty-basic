//! Mappers that are able to return an error

use crate::{parser_declaration, ParseError};
use crate::{pc::*, ParserErrorTrait};

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

// Flat map Ok and None (Error-Incomplete)
parser_declaration!(pub struct AndThenOkErr<ok_mapper: F, incomplete_mapper: G>);

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
            Err(err) => {
                if err.is_incomplete() {
                    (self.incomplete_mapper)(err)
                } else {
                    Err(err)
                }
            }
        }
    }
}
