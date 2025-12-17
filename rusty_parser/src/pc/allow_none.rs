use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};
parser_declaration!(
    pub struct AllowNoneParser {}
);

impl<I: Tokenizer + 'static, P> Parser<I> for AllowNoneParser<P>
where
    P: Parser<I>,
{
    type Output = Option<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(Some(value)),
            ParseResult::Err(err) if err.is_incomplete() => ParseResult::Ok(None),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
