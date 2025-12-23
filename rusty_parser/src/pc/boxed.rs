use crate::error::ParseError;
use crate::pc::Parser;

pub fn boxed<I, O>(parser: impl Parser<I, Output = O> + 'static) -> BoxedParser<I, O> {
    BoxedParser {
        parser: Box::new(parser),
    }
}

pub struct BoxedParser<I, O> {
    parser: Box<dyn Parser<I, Output = O>>,
}

impl<I, O> Parser<I> for BoxedParser<I, O> {
    type Output = O;

    fn parse(&self, input: I) -> super::ParseResult<I, Self::Output, ParseError> {
        self.parser.parse(input)
    }
}
