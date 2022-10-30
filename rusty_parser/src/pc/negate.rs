use crate::parser_declaration;
use crate::pc::{Parser, Tokenizer};
use rusty_common::{ParserErrorTrait, QError};
parser_declaration!(pub struct NegateParser);

impl<P> Parser for NegateParser<P>
where
    P: Parser,
{
    type Output = ();

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.parser.parse(tokenizer) {
            Ok(_) => Err(QError::Incomplete),
            Err(err) if err.is_incomplete() => Ok(()),
            Err(err) => Err(err),
        }
    }
}
