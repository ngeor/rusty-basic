//! Mappers that are able to return an error

use crate::common::QError;
use crate::parser::pc::*;
use crate::parser_declaration;

parser_declaration!(struct AndThen<mapper: F>);

impl<P, F, U> ParserBase for AndThen<P, F>
where
    P: ParserBase,
    F: Fn(P::Output) -> Result<U, QError>,
{
    type Output = U;
}

impl<P, F, U> Parser for AndThen<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).and_then(&self.mapper)
    }
}
