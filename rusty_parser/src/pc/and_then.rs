//! Mappers that are able to return an error

use crate::parser_declaration;
use crate::pc::*;
use rusty_common::*;

parser_declaration!(pub struct AndThen<mapper: F>);

impl<P, F, U> Parser for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    type Output = U;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).and_then(&self.mapper)
    }
}
