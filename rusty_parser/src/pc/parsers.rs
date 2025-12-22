use crate::pc::ParseResult;
use crate::ParseError;

/// A parser uses the given input in order to produce a result.
pub trait Parser<I> {
    type Output;

    // TODO make ParseError generic param too
    /// Parses the given input and returns a result.
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, ParseError>;
}
