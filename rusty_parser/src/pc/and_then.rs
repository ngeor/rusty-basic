//! Mappers that are able to return an error

use crate::pc::*;
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct AndThen<mapper: F>);

impl<P, F, U> Parser for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<U, ParseError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer).and_then(&self.mapper)
    }
}
