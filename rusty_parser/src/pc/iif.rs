use crate::pc::{Parser, Tokenizer};
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

impl<L, R> Parser for IIfParser<L, R>
where
    L: Parser,
    R: Parser<Output = L::Output>,
{
    type Output = L::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        if self.condition {
            self.left.parse(tokenizer)
        } else {
            self.right.parse(tokenizer)
        }
    }
}
