use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct MapIncompleteErrParser<supplier: F>);

impl<P, F> Parser for MapIncompleteErrParser<P, F>
where
    P: Parser,
    F: Fn() -> QError,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.parser.parse(tokenizer) {
            Ok(result) => Ok(result),
            Err(err) if err.is_incomplete() => Err((self.supplier)()),
            Err(err) => Err(err),
        }
    }
}
