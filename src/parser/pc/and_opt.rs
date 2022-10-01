use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};
//
// The left side can be followed by an optional right.
//
binary_parser_declaration!(struct AndOptPC);

impl<L, R> Parser for AndOptPC<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.0.parse(tokenizer)?;
        let opt_right = self.1.parse_opt(tokenizer)?;
        Ok((left, opt_right))
    }
}
