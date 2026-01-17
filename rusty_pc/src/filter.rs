use crate::{ParseResult, Parser, SetContext};

pub struct FilterParser<P, F> {
    parser: P,
    predicate: F,
}

impl<P, F> FilterParser<P, F> {
    pub fn new(parser: P, predicate: F) -> Self {
        Self { parser, predicate }
    }
}

impl<I, C, P, F> Parser<I, C> for FilterParser<P, F>
where
    I: Clone,
    P: Parser<I, C>,
    F: FilterPredicate<P::Output, P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;
    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        let original_input = input.clone();
        let (input, value) = self.parser.parse(input)?;
        match self.predicate.filter(value) {
            Ok(value) => Ok((input, value)),
            Err((fatal, err)) => Err((fatal, if fatal { input } else { original_input }, err)),
        }
    }
}

impl<C, P, F> SetContext<C> for FilterParser<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}

pub trait FilterPredicate<T, E> {
    fn filter(&self, value: T) -> Result<T, (bool, E)>;
}

impl<F, T, E> FilterPredicate<T, E> for F
where
    F: Fn(&T) -> bool,
    E: Default,
{
    fn filter(&self, value: T) -> Result<T, (bool, E)> {
        if (self)(&value) {
            Ok(value)
        } else {
            Err((false, E::default()))
        }
    }
}
