use crate::pc::{NonOptParser, Parser, Tokenizer};
use crate::{parser_declaration, ParseError, ParserErrorTrait};

parser_declaration!(pub struct AllowDefaultParser);

impl<I: Tokenizer + 'static, P> Parser<I> for AllowDefaultParser<P>
where
    P: Parser<I>,
    P::Output: Default,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err(err) if err.is_incomplete() => Ok(Self::Output::default()),
            Err(err) => Err(err),
        }
    }
}

impl<I: Tokenizer + 'static, P> NonOptParser<I> for AllowDefaultParser<P>
where
    P: Parser<I>,
    P::Output: Default,
{
}
