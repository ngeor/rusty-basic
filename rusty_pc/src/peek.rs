use crate::{ParseResult, Parser};

pub struct PeekParser<P> {
    parser: P,
}

impl<P> PeekParser<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> Parser<I, C> for PeekParser<P>
where
    I: Clone,
    P: Parser<I, C>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input.clone()) {
            Ok((_, value)) => Ok((input, value)),
            Err(err) => Err(err),
        }
    }
}
