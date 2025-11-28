use crate::pc::{NonOptParser, Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};
parser_declaration!(pub struct AllowNoneParser);

impl<I: Tokenizer + 'static, P> Parser<I> for AllowNoneParser<P>
where
    P: Parser<I>,
{
    type Output = Option<P::Output>;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.is_incomplete() => Ok(None),
            Err(err) => Err(err),
        }
    }
}
impl<I: Tokenizer + 'static, P> NonOptParser<I> for AllowNoneParser<P> where P: Parser<I> {}
