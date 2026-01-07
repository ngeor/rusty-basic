use rusty_pc::{MapErrParser, OrFail, Parser};

use crate::error::ParseError;

pub trait SpecificTrait<I>: Parser<I, Error = ParseError>
where
    Self: Sized,
{
    fn or_syntax_error(self, msg: &str) -> MapErrParser<Self, ParseError> {
        self.or_fail(ParseError::syntax_error(msg))
    }

    /// Demands a successful result or returns a fatal syntax error
    /// with an error message like "Expected: " followed by the
    /// given expectation message.
    fn or_expected(self, expectation: &str) -> MapErrParser<Self, ParseError> {
        self.or_fail(ParseError::expected(expectation))
    }
}

impl<I, P> SpecificTrait<I> for P where P: Parser<I, Error = ParseError> {}
