use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};
parser_declaration!(pub struct NegateParser);

impl<P> Parser for NegateParser<P>
where
    P: Parser,
{
    type Output = ();

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(_) => Err(ParseError::Incomplete),
            Err(err) if err.is_incomplete() => Ok(()),
            Err(err) => Err(err),
        }
    }
}
