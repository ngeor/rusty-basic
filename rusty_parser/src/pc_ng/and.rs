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
    R: Parser<Input = L::Input, Error = L::Error>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Input = L::Input;
    type Output = O;
    type Error = L::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.left.parse(input).flat_map(|i, left_result| {
            self.right
                .parse(i)
                .map_ok(|right_result| (self.combiner)(left_result, right_result))
        })
    }
}
