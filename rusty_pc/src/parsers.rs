use crate::{FilterParser, FilterPredicate, ParseResult};

/// A parser uses the given input in order to produce a result.
pub trait Parser<I, C = ()> {
    type Output;
    type Error: ParserErrorTrait;

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

pub trait ParserErrorTrait: Clone + Default {
    fn is_fatal(&self) -> bool;

    fn to_fatal(self) -> Self;
}
