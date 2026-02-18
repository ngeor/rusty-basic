use rusty_pc::map_soft_err::MapSoftErrParser;
use rusty_pc::{Parser, ParserErrorTrait};

use crate::error::ParserError;
use crate::input::StringView;

pub trait OrExpected<C>: Parser<StringView, C, Error = ParserError>
where
    Self: Sized,
{
    /// Demands a successful result or returns a fatal syntax error
    /// with an error message like "Expected: " followed by the
    /// given expectation message.
    fn or_expected(self, expectation: &str) -> MapSoftErrParser<Self, Self::Error> {
        self.or_fail(ParserError::expected(expectation).to_fatal())
    }
}

impl<P, C> OrExpected<C> for P where P: Parser<StringView, C, Error = ParserError> {}
