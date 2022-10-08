use crate::common::QError;
use crate::parser::pc::{Parser, ParserOnce, Tokenizer};
use crate::parser_declaration;

parser_declaration!(pub struct MapOnce<mapper: F>);

impl<P, F, U> ParserOnce for MapOnce<P, F>
where
    P: Parser,
    F: FnOnce(P::Output) -> U,
{
    type Output = U;

    fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.parser.parse(tokenizer) {
            Ok(result) => Ok((self.mapper)(result)),
            Err(err) => Err(err),
        }
    }
}
