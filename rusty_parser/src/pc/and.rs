use crate::pc::{ParseResult, Parser, Tokenizer, Undo};
use crate::{binary_parser_declaration, ParseError};

//
// And (with undo if the left parser supports it)
//

binary_parser_declaration!(pub struct AndPC);

impl<I: Tokenizer + 'static, A, B> Parser<I> for AndPC<A, B>
where
    A: Parser<I>,
    A::Output: Undo,
    B: Parser<I>,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.left.parse(tokenizer) {
            ParseResult::Ok(left) => match self.right.parse_opt(tokenizer) {
                ParseResult::Ok(Some(right)) => ParseResult::Ok((left, right)),
                ParseResult::Ok(None) => {
                    left.undo(tokenizer);
                    ParseResult::Err(ParseError::Incomplete)
                }
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
