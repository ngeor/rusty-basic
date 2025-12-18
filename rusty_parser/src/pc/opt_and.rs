use crate::pc::{ParseResult, Parser, Token, Tokenizer, Undo};
use crate::{binary_parser_declaration, ParseError};

// The left side is optional, the right is not.
// If the right is missing, the left is reverted.

binary_parser_declaration!(pub struct OptAndPC);

impl<I: Tokenizer + 'static, L, R> Parser<I> for OptAndPC<L, R>
where
    L: Parser<I, Output = Token>,
    R: Parser<I>,
{
    type Output = (Option<L::Output>, R::Output);
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.left.parse(tokenizer) {
            ParseResult::Ok(opt_leading) => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((Some(opt_leading), right)),
                ParseResult::None => {
                    opt_leading.undo(tokenizer);
                    ParseResult::None
                }
                ParseResult::Expected(s) => {
                    opt_leading.undo(tokenizer);
                    ParseResult::Expected(s)
                }
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::None | ParseResult::Expected(_) => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((None, right)),
                ParseResult::None => ParseResult::None,
                ParseResult::Expected(s) => ParseResult::Expected(s),
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
