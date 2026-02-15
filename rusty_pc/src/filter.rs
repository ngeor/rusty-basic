use crate::{InputTrait, Parser, default_parse_error};

pub struct FilterParser<P, F> {
    parser: P,
    predicate: F,
}

impl<P, F> FilterParser<P, F> {
    pub(crate) fn new(parser: P, predicate: F) -> Self {
        Self { parser, predicate }
    }
}

impl<I, C, P, F> Parser<I, C> for FilterParser<P, F>
where
    I: InputTrait,
    I: InputTrait,
    P: Parser<I, C>,
    F: Fn(&P::Output) -> bool,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let original_input = input.get_position();
        let value = self.parser.parse(input)?;
        if (self.predicate)(&value) {
            Ok(value)
        } else {
            input.set_position(original_input);
            default_parse_error()
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.parser.set_context(ctx)
    }
}
