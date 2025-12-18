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
            ParseResult::Ok(left) => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((left, right)),
                ParseResult::None => {
                    left.undo(tokenizer);
                    ParseResult::None
                }
                ParseResult::Expected(s) => {
                    left.undo(tokenizer);
                    ParseResult::Expected(s)
                }
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::None => ParseResult::None,
            ParseResult::Expected(s) => ParseResult::Expected(s),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}

binary_parser_declaration!(pub struct AndWithoutUndoPC);

impl<I: Tokenizer + 'static, A, B> Parser<I> for AndWithoutUndoPC<A, B>
where
    A: Parser<I>,
    B: Parser<I>,
{
    type Output = (A::Output, B::Output);

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.left
            .parse(tokenizer)
            .flat_map(|left| self.right.parse(tokenizer).map(|right| (left, right)))
    }
}
