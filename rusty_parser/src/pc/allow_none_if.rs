use crate::pc::{Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};
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
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.is_incomplete() && self.condition => Ok(None),
            Err(err) => Err(err),
        }
    }
}
