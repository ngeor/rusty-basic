use crate::pc::{Parser, ParserOnce, Tokenizer};
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct ChainParser);

impl<I: Tokenizer + 'static, L, RF, R> Parser<I> for ChainParser<L, RF>
where
    L: Parser<I>,
    RF: Fn(L::Output) -> R,
    R: ParserOnce<I>,
{
    type Output = <R as ParserOnce<I>>::Output;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        let first = self.left.parse(tokenizer)?;
        let right_parser = (self.right)(first);
        right_parser.parse(tokenizer)
    }
}
