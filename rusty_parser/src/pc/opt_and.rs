use crate::pc::{ParseResult, Parser, Token};
use crate::{binary_parser_declaration, ParseError};

// The left side is optional, the right is not.
// If the right is missing, the left is reverted.

binary_parser_declaration!(pub struct OptAndPC);

impl<I: Clone, L, R> Parser<I> for OptAndPC<L, R>
where
    L: Parser<I, Output = Token>,
    R: Parser<I>,
{
    type Output = (Option<L::Output>, R::Output);
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.left.parse(tokenizer.clone()) {
            Ok((input, opt_leading)) => match self.right.parse(input) {
                Ok((input, right)) => Ok((input, (Some(opt_leading), right))),
                // right side failed, reverting left too
                Err((false, _, err)) => Err((false, tokenizer, err)),
                Err(err) => Err(err),
            },
            Err((false, input, _)) => match self.right.parse(input) {
                Ok((input, right)) => Ok((input, (None, right))),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }
}
