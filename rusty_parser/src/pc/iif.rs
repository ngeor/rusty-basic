use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::ParseError;

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

impl<I: Tokenizer + 'static, L, R> Parser<I> for IIfParser<L, R>
where
    L: Parser<I>,
    R: Parser<I, Output = L::Output>,
{
    type Output = L::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        if self.condition {
            self.left.parse(tokenizer)
        } else {
            self.right.parse(tokenizer)
        }
    }
}
