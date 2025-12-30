use crate::ParseResult;
use crate::map_err::MapErrParser;
use crate::or_default::OrDefaultParser;

/// A parser uses the given input in order to produce a result.
pub trait Parser<I> {
    type Output;
    type Error;

    /// Parses the given input and returns a result.
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error>;

    fn or_default(self) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        Self::Output: Default,
    {
        OrDefaultParser::new(self)
    }

    fn no_incomplete(self) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        Self::Error: Clone,
    {
        MapErrParser::new(self).no_incomplete()
    }

    fn or_fail(self, err: Self::Error) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        Self::Error: Clone,
    {
        MapErrParser::new(self).or_fail(err)
    }
}
