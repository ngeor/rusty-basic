use crate::pc_ng::*;

pub struct Filter<P, F> {
    parser: P,
    predicate: F,
}

impl<P, F> Filter<P, F> {
    pub fn new(parser: P, predicate: F) -> Self {
        Filter { parser, predicate }
    }
}

impl<P, F> Parser for Filter<P, F>
where
    P: Parser,
    P::Input: Clone,
    F: Fn(&P::Output) -> bool,
{
    type Input = P::Input;
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.parser.parse(input.clone()).flat_map(|i, o| {
            if (self.predicate)(&o) {
                ParseResult::Ok(i, o)
            } else {
                // return original input here
                ParseResult::None(input)
            }
        })
    }
}
