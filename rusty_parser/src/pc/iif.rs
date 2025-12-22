use crate::pc::{ParseResult, Parser};
use crate::ParseError;

#[deprecated]
pub fn iif_p<L, R>(condition: bool, left: L, right: R) -> IIfParser<L, R> {
    IIfParser::new(condition, left, right)
}

pub struct IIfParser<L, R> {
    condition: bool,
    left: L,
    right: R,
}

impl<L, R> IIfParser<L, R> {
    pub fn new(condition: bool, left: L, right: R) -> Self {
        Self {
            condition,
            left,
            right,
        }
    }
}

impl<I, L, R> Parser<I> for IIfParser<L, R>
where
    L: Parser<I>,
    R: Parser<I, Output = L::Output>,
{
    type Output = L::Output;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        if self.condition {
            self.left.parse(tokenizer)
        } else {
            self.right.parse(tokenizer)
        }
    }
}
