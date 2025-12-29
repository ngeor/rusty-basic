use crate::pc::{default_parse_error, ParseResult, ParseResultTrait, Parser};

pub trait FilterMap<I>: Parser<I>
where
    Self: Sized,
    Self::Error: Default,
    I: Clone,
{
    fn filter_map<F, U>(
        self,
        predicate_mapper: F,
    ) -> impl Parser<I, Output = U, Error = Self::Error>
    where
        F: Fn(&Self::Output) -> Option<U>,
    {
        FilterMapParser(self, predicate_mapper)
    }
}

impl<I, P> FilterMap<I> for P
where
    I: Clone,
    P: Parser<I>,
    P::Error: Default,
{
}

struct FilterMapParser<P, F>(P, F);

impl<I: Clone, P, F, U> Parser<I> for FilterMapParser<P, F>
where
    P: Parser<I>,
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
