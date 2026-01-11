use rusty_pc::{OrFail, Parser, SurroundMode, ToFatalParser, surround};

use crate::error::ParseError;
use crate::input::RcStringView;
use crate::tokens::whitespace;

pub trait SpecificTrait: Parser<RcStringView, Error = ParseError>
where
    Self: Sized,
{
    fn or_syntax_error(self, msg: &str) -> ToFatalParser<Self, ParseError> {
        self.or_fail(ParseError::syntax_error(msg))
    }

    /// Demands a successful result or returns a fatal syntax error
    /// with an error message like "Expected: " followed by the
    /// given expectation message.
    fn or_expected(self, expectation: &str) -> ToFatalParser<Self, ParseError> {
        self.or_fail(ParseError::expected(expectation))
    }

    fn padded_by_ws(self) -> impl Parser<RcStringView, Output = Self::Output, Error = Self::Error> {
        surround(whitespace(), self, whitespace(), SurroundMode::Optional)
    }
}

impl<P> SpecificTrait for P where P: Parser<RcStringView, Error = ParseError> {}
