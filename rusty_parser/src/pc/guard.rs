use crate::pc::*;
use crate::{binary_parser_declaration, ParseError};

binary_parser_declaration!(pub struct GuardPC);

impl<L, R> Parser for GuardPC<L, R>
where
    L: Parser,
    R: Parser + NonOptParser,
{
    type Output = <R as Parser>::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        self.left.parse(tokenizer)?;
        self.right.parse(tokenizer)
    }
}
