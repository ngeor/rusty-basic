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
    /// Gets a value indicating whether this is a fatal error or not.
    /// Returns true if the error is fatal, false is the error is soft.
    fn is_fatal(&self) -> bool;

    /// Converts this error into a fatal.
    fn to_fatal(self) -> Self;
}
