use crate::binary_parser_declaration;
use crate::parser::pc::{Parser, Tokenizer, Undo};
use rusty_common::*;

//
// And (with undo if the left parser supports it)
//

// Looks identical to `NonOptSeq2` but that one has already an implementation
// of Parser

binary_parser_declaration!(pub struct AndPC);

impl<A, B> Parser for AndPC<A, B>
where
    A: Parser,
    A::Output: Undo,
    B: Parser,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.left.parse(tokenizer)?;
        if let Some(right) = self.right.parse_opt(tokenizer)? {
            Ok((left, right))
        } else {
            left.undo(tokenizer);
            Err(QError::Incomplete)
        }
    }
}
