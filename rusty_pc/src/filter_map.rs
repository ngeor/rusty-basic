use crate::{ParseResult, ParseResultTrait, Parser, SetContext, default_parse_error};

pub trait FilterMap<I, C>: Parser<I, C>
where
    Self: Sized,
    I: Clone,
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
    P: Parser<I, C>,
    I: Clone,
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
    P: Parser<I, C>,
    I: Clone,
    P::Error: Default,
    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;
    type Error = P::Error;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parser
            .parse(tokenizer.clone())
            .flat_map(|input, result| match (self.predicate)(&result) {
                Some(value) => Ok((input, value)),
                None => default_parse_error(tokenizer),
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
