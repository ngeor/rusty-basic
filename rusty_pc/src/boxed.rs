use crate::{Parser, ParserErrorTrait};

pub fn boxed<I, C, O, E>(
    parser: impl Parser<I, C, Output = O, Error = E> + 'static,
) -> BoxedParser<I, C, O, E>
where
    E: ParserErrorTrait,
{
    BoxedParser {
        parser: Box::new(parser),
    }
}

pub struct BoxedParser<I, C, O, E> {
    parser: Box<dyn Parser<I, C, Output = O, Error = E>>,
}

impl<I, C, O, E> Parser<I, C> for BoxedParser<I, C, O, E>
where
    E: ParserErrorTrait,
{
    type Output = O;
    type Error = E;

    fn parse(&mut self, input: I) -> super::ParseResult<I, Self::Output, Self::Error> {
        self.parser.parse(input)
    }
}
