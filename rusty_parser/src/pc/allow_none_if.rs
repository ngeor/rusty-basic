use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};
parser_declaration!(
    pub struct AllowNoneIfParser {
        condition: bool,
    }
);

impl<I: Tokenizer + 'static, P> Parser<I> for AllowNoneIfParser<P>
where
    P: Parser<I>,
{
    type Output = Option<P::Output>;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(Some(value)),
            ParseResult::None => {
                if self.condition {
                    ParseResult::Ok(None)
                } else {
                    ParseResult::None
                }
            }
            ParseResult::Expected(s) => {
                if self.condition {
                    ParseResult::Ok(None)
                } else {
                    ParseResult::Expected(s)
                }
            }
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
