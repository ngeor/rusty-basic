use crate::ParseResult;

/// A parser uses the given input in order to produce a result.
pub trait Parser<I, C = ()> {
    type Output;
    type Error;

    /// Parses the given input and returns a result.
    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error>;

    /// Creates a new parser where the context is set to the given value.
    /// A parser that can store context needs to override this method.
    /// Parsers that delegate to other parsers should implement this method
    /// by propagating the context to the delegate.
    fn set_context(&mut self, ctx: C);
}
