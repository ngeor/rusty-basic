use crate::pc_ng::*;

pub struct Opt<P> {
    parser: P,
}

impl<P> Opt<P> {
    pub fn new(parser: P) -> Self {
        Opt { parser }
    }
}

impl<P> Parser for Opt<P>
where
    P: Parser,
    P::Input: Clone,
{
    type Input = P::Input;
    type Output = Option<P::Output>;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        match self.parser.parse(input.clone()) {
            Ok((i, o)) => Ok((i, Some(o))),
            Err((false, _)) => Ok((input, None)),
            Err(e) => Err(e),
        }
    }
}
