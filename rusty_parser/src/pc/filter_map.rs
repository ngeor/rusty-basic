use crate::pc::{ParseResult, Parser, Tokenizer, Undo};
use crate::ParseError;

pub trait FilterMap<I: Tokenizer + 'static>: Parser<I> {
    fn filter_map<F, U>(self, predicate_mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        Self::Output: Undo,
        F: Fn(&Self::Output) -> Option<U>;
}

impl<I, P> FilterMap<I> for P
where
    I: Tokenizer + 'static,
    P: Parser<I>,
    P::Output: Undo,
{
    fn filter_map<F, U>(self, predicate_mapper: F) -> impl Parser<I, Output = U>
    where
        Self: Sized,
        Self::Output: Undo,
        F: Fn(&Self::Output) -> Option<U>,
    {
        FilterMapParser(self, predicate_mapper)
    }
}

struct FilterMapParser<P, F>(P, F);

impl<I: Tokenizer + 'static, P, F, U> Parser<I> for FilterMapParser<P, F>
where
    P: Parser<I>,
    P::Output: Undo,
    F: Fn(&P::Output) -> Option<U>,
{
    type Output = U;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.0
            .parse(tokenizer)
            .flat_map(|result| match (self.1)(&result) {
                Some(value) => ParseResult::Ok(value),
                None => {
                    result.undo(tokenizer);
                    ParseResult::None
                }
            })
    }
}
