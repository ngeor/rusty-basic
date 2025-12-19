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
    F: Fn(&P::Output) -> bool,
    P::Error: Default,
{
    type Input = P::Input;
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.parser.parse(input).filter(&self.predicate)
    }
}
