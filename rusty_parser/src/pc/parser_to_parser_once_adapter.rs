use crate::pc::{Parser, ParserOnce, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct ParserToParserOnceAdapter);

impl<P> ParserOnce for ParserToParserOnceAdapter<P>
where
    P: Parser,
{
    type Output = P::Output;

    fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer)
    }
}
