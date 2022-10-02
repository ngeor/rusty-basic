use crate::binary_parser_declaration;
use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::*;

binary_parser_declaration!(pub struct GuardPC);

impl<L, R> Parser for GuardPC<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = R::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.left.parse(tokenizer)?;
        self.right.parse(tokenizer).no_incomplete()
    }
}
