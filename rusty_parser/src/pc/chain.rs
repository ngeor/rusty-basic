use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::ParseError;

pub struct ChainParser<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> ChainParser<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I: Tokenizer + 'static, L, RF, R, F, O> Parser<I> for ChainParser<L, RF, F>
where
    L: Parser<I>,
    RF: Fn(&L::Output) -> R,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.left.parse(tokenizer) {
            ParseResult::Ok(first) => {
                let right_parser = (self.right)(&first);
                right_parser
                    .parse(tokenizer)
                    .map(|r| (self.combiner)(first, r))
            }
            ParseResult::Err(e) => ParseResult::Err(e),
        }
    }
}
