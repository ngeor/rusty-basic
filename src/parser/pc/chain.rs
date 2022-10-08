use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{Parser, ParserOnce, Tokenizer};

binary_parser_declaration!(pub struct ChainParser);

impl<L, RF, R> Parser for ChainParser<L, RF>
where
    L: Parser,
    RF: Fn(L::Output) -> R,
    R: ParserOnce,
{
    type Output = <R as ParserOnce>::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.left.parse(tokenizer)?;
        let right_parser = (self.right)(first);
        right_parser.parse(tokenizer)
    }
}
