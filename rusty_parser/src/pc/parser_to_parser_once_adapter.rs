use crate::parser_declaration;
use crate::pc::{Parser, ParserOnce, Tokenizer};
use rusty_common::*;

parser_declaration!(pub struct ParserToParserOnceAdapter);

impl<P> ParserOnce for ParserToParserOnceAdapter<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer)
    }
}
