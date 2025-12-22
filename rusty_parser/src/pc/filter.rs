use crate::pc::{default_parse_error, ParseResult, Parser};
use crate::ParseError;

pub trait Filter<I>: Parser<I>
where
    Self: Sized,
    I: Clone,
{
    fn filter<F>(self, predicate: F) -> impl Parser<I, Output = Self::Output>
    where
        F: Fn(&Self::Output) -> bool,
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
    F: Fn(&P::Output) -> bool,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
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
