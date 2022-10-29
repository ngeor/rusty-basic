use crate::parser::pc::{NonOptParser, Parser, Tokenizer};
use crate::parser_declaration;
use rusty_common::{ParserErrorTrait, QError};
parser_declaration!(pub struct AllowNoneParser);

impl<P> Parser for AllowNoneParser<P>
where
    P: Parser,
{
    type Output = Option<P::Output>;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.is_incomplete() => Ok(None),
            Err(err) => Err(err),
        }
    }
}
impl<P> NonOptParser for AllowNoneParser<P> where P: Parser {}
