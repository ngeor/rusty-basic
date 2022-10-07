use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{NonOptParser, Parser, Tokenizer};
binary_parser_declaration!(pub struct PipeParser);

impl<L, RF, R> Parser for PipeParser<L, RF>
where
    L: Parser,
    RF: Fn(&L::Output) -> R,
    R: Parser + NonOptParser,
{
    type Output = (L::Output, <R as Parser>::Output);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.left.parse(tokenizer)?;
        let right_parser = (self.right)(&left);
        let right = right_parser.parse(tokenizer)?;
        Ok((left, right))
    }
}
