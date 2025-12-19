use crate::pc_ng::*;

pub struct Map<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F> Map<P, F> {
    pub fn new(parser: P, mapper: F) -> Self {
        Map { parser, mapper }
    }
}

impl<P, F, O> Parser for Map<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> O,
{
    type Input = P::Input;
    type Output = O;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        self.parser.parse(input).map_ok(&self.mapper)
    }
}
