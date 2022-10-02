use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::{NonOptParser, Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct AllowDefaultParser);

impl<P> Parser for AllowDefaultParser<P>
where
    P: Parser,
    P::Output: Default,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err(err) if err.is_incomplete() => Ok(Self::Output::default()),
            Err(err) => Err(err),
        }
    }
}

impl<P> NonOptParser for AllowDefaultParser<P>
where
    P: Parser,
    P::Output: Default,
{
}
