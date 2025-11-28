use crate::pc::{Parser, Tokenizer, Undo};
use crate::{binary_parser_declaration, ParseError};

//
// And (with undo if the left parser supports it)
//

// Looks identical to `NonOptSeq2` but that one has already an implementation
// of Parser

binary_parser_declaration!(pub struct AndPC);

impl<I: Tokenizer + 'static, A, B> Parser<I> for AndPC<A, B>
where
    A: Parser<I>,
    A::Output: Undo,
    B: Parser<I>,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        let left = self.left.parse(tokenizer)?;
        if let Some(right) = self.right.parse_opt(tokenizer)? {
            Ok((left, right))
        } else {
            left.undo(tokenizer);
            Err(ParseError::Incomplete)
        }
    }
}
