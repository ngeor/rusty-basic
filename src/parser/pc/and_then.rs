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

impl<P, F, U> OptParser for AndThen<P, F>
where
    P: OptParser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(value) => (self.mapper)(value).map(Some),
            None => Ok(None),
        }
    }
}

impl<P, F, U> NonOptParser for AndThen<P, F>
where
    P: NonOptParser,
    F: Fn(P::Output) -> Result<U, QError>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse_non_opt(tokenizer).and_then(&self.mapper)
    }
}
