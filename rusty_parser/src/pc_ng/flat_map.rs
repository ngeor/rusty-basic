use crate::pc_ng::*;

pub struct FlatMap<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> FlatMap<P, F> {
    pub fn new(parser: P, mapper: F) -> Self {
        FlatMap { parser, mapper }
    }
}

impl<P, F, O> Parser for FlatMap<P, F>
where
    P: Parser,
    F: Fn(P::Input, P::Output) -> ParseResult<P::Input, O, P::Error>,
{
    type Input = P::Input;
    type Output = O;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.parser.parse(input).flat_map(&self.mapper)
    }
}
