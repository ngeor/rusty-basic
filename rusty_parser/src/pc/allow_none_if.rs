use crate::pc::{ParseResult, Parser};
use crate::{parser_declaration, ParseError};
parser_declaration!(
    pub struct AllowNoneIfParser {
        condition: bool,
    }
);

impl<I, P> Parser<I> for AllowNoneIfParser<P>
where
    P: Parser<I>,
{
    type Output = Option<P::Output>;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok((input, value)) => Ok((input, Some(value))),
            Err((fatal, i, err)) => {
                if self.condition && !fatal {
                    Ok((i, None))
                } else {
                    Err((fatal, i, err))
                }
            }
        }
    }
}
