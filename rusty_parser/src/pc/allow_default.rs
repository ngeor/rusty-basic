use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct AllowDefaultParser);

impl<I: Tokenizer + 'static, P> Parser<I> for AllowDefaultParser<P>
where
    P: Parser<I>,
    P::Output: Default,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None | ParseResult::Expected(_) => {
                ParseResult::Ok(Self::Output::default())
            }
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
