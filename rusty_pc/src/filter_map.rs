use crate::{ParseResult, ParseResultTrait, Parser, default_parse_error};

pub trait FilterMap<I, C>: Parser<I, C>
where
    Self: Sized,
    Self::Error: Default,
    I: Clone,
{
    fn filter_map<F, U>(
        self,
        predicate_mapper: F,
    ) -> impl Parser<I, C, Output = U, Error = Self::Error>
    where
        F: Fn(&Self::Output) -> Option<U>,
    {
        FilterMapParser(self, predicate_mapper)
    }
}

impl<I, C, P> FilterMap<I, C> for P
where
    I: Clone,
    P: Parser<I, C>,
    P::Error: Default,
{
}

struct FilterMapParser<P, F>(P, F);

impl<I: Clone, C, P, F, U> Parser<I, C> for FilterMapParser<P, F>
where
    P: Parser<I, C>,
    P::Error: Default,
    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.0
            .parse(tokenizer.clone())
            .flat_map(|input, result| match (self.1)(&result) {
                Some(value) => Ok((input, value)),
                None => default_parse_error(tokenizer),
            })
    }
}
