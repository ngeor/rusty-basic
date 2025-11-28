use crate::pc::{Parser, ParserOnce, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct ParserToParserOnceAdapter);

impl<I: Tokenizer + 'static, P> ParserOnce<I> for ParserToParserOnceAdapter<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        self.parser.parse(tokenizer)
    }
}
