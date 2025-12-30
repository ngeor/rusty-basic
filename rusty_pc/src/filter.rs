use crate::{ParseResult, Parser, default_parse_error};

pub trait Filter<I>: Parser<I>
where
    Self: Sized,
    I: Clone,
{
    fn filter<F>(self, predicate: F) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        F: Fn(&Self::Output) -> bool,
        Self::Error: Default,
    {
        FilterParser(self, predicate)
    }
}

impl<I, P> Filter<I> for P
where
    I: Clone,
    P: Parser<I>,
{
}

struct FilterParser<P, F>(P, F);

impl<I, P, F> Parser<I> for FilterParser<P, F>
where
    I: Clone,
    P: Parser<I>,
    P::Error: Default,
    F: Fn(&P::Output) -> bool,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.0.parse(tokenizer.clone()) {
            Ok((input, value)) => {
                if (self.1)(&value) {
                    Ok((input, value))
                } else {
                    // return original input here
                    default_parse_error(tokenizer)
                }
            }
            Err(err) => Err(err),
        }
    }
}
