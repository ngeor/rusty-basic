use crate::pc_ng::*;

pub struct And<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}

impl<L, R, F> And<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<L, R, F, O> Parser for And<L, R, F>
where
    L: Parser,
    L::Input: Clone,
    R: Parser<Input = L::Input, Error = L::Error>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Input = L::Input;
    type Output = O;
    type Error = L::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.left.parse(input.clone()).flat_map(|i, left_result| {
            match self.right.parse(i) {
                ParseResult::Ok(i, right_result) => {
                    ParseResult::Ok(i, (self.combiner)(left_result, right_result))
                }
                // return original input here
                ParseResult::None(_) => ParseResult::None(input),
                ParseResult::Err(i, err) => ParseResult::Err(i, err),
            }
        })
    }
}
