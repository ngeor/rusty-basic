use crate::{InputTrait, Parser, ParserErrorTrait};

pub struct BoxedParser<I, C, O, E> {
    parser: Box<dyn Parser<I, C, Output = O, Error = E>>,
}

impl<I, C, O, E> BoxedParser<I, C, O, E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    pub(crate) fn new(parser: impl Parser<I, C, Output = O, Error = E> + 'static) -> Self {
        BoxedParser {
            parser: Box::new(parser),
        }
    }
}

impl<I, C, O, E> Parser<I, C> for BoxedParser<I, C, O, E>
where
    I: InputTrait,
    E: ParserErrorTrait,
{
    type Output = O;
    type Error = E;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        self.parser.parse(input)
    }

    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
