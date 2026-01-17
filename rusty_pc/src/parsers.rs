use crate::{FilterParser, FilterPredicate, ParseResult};

/// A parser uses the given input in order to produce a result.
pub trait Parser<I, C = ()> {
    type Output;
    type Error;

    /// Parses the given input and returns a result.
    fn parse(&mut self, input: I) -> ParseResult<I, Self::Output, Self::Error>;

    fn filter<F>(self, predicate: F) -> FilterParser<Self, F>
    where
        Self: Sized,
        F: FilterPredicate<Self::Output, Self::Error>,
    {
        FilterParser::new(self, predicate)
    }
}
