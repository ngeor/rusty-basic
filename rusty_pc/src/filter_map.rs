use crate::{InputTrait, Parser, SetContext, default_parse_error};

pub trait FilterMap<I: InputTrait, C>: Parser<I, C>
where
    Self: Sized,
    I: InputTrait,
    Self::Error: Default,
{
    fn filter_map<F, U>(self, predicate: F) -> impl Parser<I, C, Output = U, Error = Self::Error>
    where
        F: Fn(&Self::Output) -> Option<U>,
    {
        FilterMapParser::new(self, predicate)
    }
}
impl<I, C, P> FilterMap<I, C> for P
where
    I: InputTrait,
    P: Parser<I, C>,
    I: InputTrait,
    P::Error: Default,
{
}

struct FilterMapParser<P, F> {
    parser: P,
    predicate: F,
}
impl<P, F> FilterMapParser<P, F> {
    pub fn new(parser: P, predicate: F) -> Self {
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
}
impl<C, P, F> SetContext<C> for FilterMapParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
