use crate::pc::{Parser, ParserOnce, Tokenizer};
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct ChainParser);

impl<L, RF, R> Parser for ChainParser<L, RF>
where
    L: Parser,
    RF: Fn(L::Output) -> R,
    R: ParserOnce,
{
    type Output = <R as ParserOnce>::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let first = self.left.parse(tokenizer)?;
        let right_parser = (self.right)(first);
        right_parser.parse(tokenizer)
    }
}
