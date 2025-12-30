use crate::or_default::OrDefaultParser;
use crate::{NoIncompleteParser, ParseResult};

/// A parser uses the given input in order to produce a result.
pub trait Parser<I> {
    type Output;
    type Error;

    /// Parses the given input and returns a result.
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error>;

    fn no_incomplete(self) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
    {
        NoIncompleteParser::new(self)
    }

    fn or_default(self) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        Self::Output: Default,
    {
        OrDefaultParser::new(self)
    }
}
