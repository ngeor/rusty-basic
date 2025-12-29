use crate::pc::Parser;

pub fn boxed<I, O, E>(
    parser: impl Parser<I, Output = O, Error = E> + 'static,
) -> BoxedParser<I, O, E> {
    BoxedParser {
        parser: Box::new(parser),
    }
}

pub struct BoxedParser<I, O, E> {
    parser: Box<dyn Parser<I, Output = O, Error = E>>,
}

impl<I, O, E> Parser<I> for BoxedParser<I, O, E> {
    type Output = O;
    type Error = E;

    fn parse(&self, input: I) -> super::ParseResult<I, Self::Output, Self::Error> {
        self.parser.parse(input)
    }
}
