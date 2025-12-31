use crate::ParseResult;

/// A parser uses the given input in order to produce a result.
pub trait Parser<I> {
    type Output;
    type Error;

    /// Parses the given input and returns a result.
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error>;
}
