use crate::pc::{default_parse_error, ParseResult, ParseResultTrait, Parser};
use crate::ParseError;

pub trait FilterMap<I: Clone>: Parser<I> {
    fn filter_map<F, U>(self, predicate_mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,

        F: Fn(&Self::Output) -> Option<U>;
}

impl<I: Clone, P> FilterMap<I> for P
where
    P: Parser<I>,
{
    fn filter_map<F, U>(self, predicate_mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,

        F: Fn(&Self::Output) -> Option<U>,
    {
        FilterMapParser(self, predicate_mapper)
    }
}

struct FilterMapParser<P, F>(P, F);

impl<I: Clone, P, F, U> Parser<I> for FilterMapParser<P, F>
where
    P: Parser<I>,

    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        self.0
            .parse(tokenizer.clone())
            .flat_map(|input, result| match (self.1)(&result) {
                Some(value) => Ok((input, value)),
                None => default_parse_error(tokenizer),
            })
    }
}
