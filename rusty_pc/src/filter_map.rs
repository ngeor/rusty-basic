use crate::{InputTrait, Parser, default_parse_error};

pub struct FilterMapParser<P, F> {
    parser: P,
    predicate: F,
}
impl<P, F> FilterMapParser<P, F> {
    pub(crate) fn new(parser: P, predicate: F) -> Self {
        Self { parser, predicate }
    }
}
impl<I, C, P, F, U> Parser<I, C> for FilterMapParser<P, F>
where
    I: InputTrait,
    P: Parser<I, C>,
    P::Error: Default,
    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&mut self, tokenizer: &mut I) -> Result<Self::Output, Self::Error> {
        let original_input = tokenizer.get_position();
        self.parser
            .parse(tokenizer)
            .and_then(|result| match (self.predicate)(&result) {
                Some(value) => Ok(value),
                None => {
                    tokenizer.set_position(original_input);
                    default_parse_error()
                }
            })
    }

    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
