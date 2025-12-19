use crate::pc::{ParseResult, Parser, Tokenizer, Undo};
use crate::ParseError;

pub trait Filter<I: Tokenizer + 'static>: Parser<I> {
    fn filter<F>(self, predicate: F) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        Self::Output: Undo,
        F: Fn(&Self::Output) -> bool;
}

impl<I, P> Filter<I> for P
where
    I: Tokenizer + 'static,
    P: Parser<I>,
    P::Output: Undo,
{
    fn filter<F>(self, predicate: F) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        Self::Output: Undo,
        F: Fn(&Self::Output) -> bool,
    {
        FilterParser(self, predicate)
    }
}

struct FilterParser<P, F>(P, F);

impl<I: Tokenizer + 'static, P, F> Parser<I> for FilterParser<P, F>
where
    P: Parser<I>,
    F: Fn(&P::Output) -> bool,
    P::Output: Undo,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.0.parse(tokenizer).flat_map(|result| {
            if (self.1)(&result) {
                ParseResult::Ok(result)
            } else {
                result.undo(tokenizer);
                ParseResult::None
            }
        })
    }
}
