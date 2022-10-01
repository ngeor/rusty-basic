use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::*;

binary_parser_declaration!(struct GuardPC);

impl<L, R> Parser for GuardPC<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = R::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse(tokenizer)?;
        self.1.parse(tokenizer)
    }
}
