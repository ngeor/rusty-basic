use crate::pc::{ParseResult, Parser, Token, Tokenizer, Undo};
use crate::{binary_parser_declaration, ParseError, ParserErrorTrait};

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
        match self.left.parse_opt(tokenizer) {
            ParseResult::Ok(opt_leading) => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((opt_leading, right)),
                ParseResult::None => {
                    opt_leading.undo(tokenizer);
                    ParseResult::None
                }
                ParseResult::Err(err) if err.is_incomplete() => {
                    opt_leading.undo(tokenizer);
                    ParseResult::Err(err)
                }
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::None => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((None, right)),
                ParseResult::None => ParseResult::None,
                ParseResult::Err(err) if err.is_incomplete() => ParseResult::Err(err),
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
