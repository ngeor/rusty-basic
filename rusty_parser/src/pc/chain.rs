use crate::pc::{ParseResult, ParseResultTrait, Parser};
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

impl<I, L, RF, R, F, O> Parser<I> for ChainParser<L, RF, F>
where
    L: Parser<I>,
    RF: Fn(&L::Output) -> R,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        self.left.parse(tokenizer).flat_map(|tokenizer: I, first| {
            let right_parser = (self.right)(&first);
            right_parser
                .parse(tokenizer)
                .map_ok(|r| (self.combiner)(first, r))
        })
    }
}
