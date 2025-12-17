use crate::pc::*;
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct GuardPC);

impl<I: Tokenizer + 'static, L, R> Parser<I> for GuardPC<L, R>
where
    L: Parser<I>,
    R: Parser<I>,
{
    type Output = <R as Parser<I>>::Output;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.left
            .parse(tokenizer)
            .flat_map(|_| self.right.parse(tokenizer))
    }
}
