use crate::pc::{Parser, Tokenizer};
use crate::{binary_parser_declaration, ParseError};
//
// The left side can be followed by an optional right.
//
binary_parser_declaration!(pub struct AndOptPC);

impl<L, R> Parser for AndOptPC<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let left = self.left.parse(tokenizer)?;
        let opt_right = self.right.parse_opt(tokenizer)?;
        Ok((left, opt_right))
    }
}
