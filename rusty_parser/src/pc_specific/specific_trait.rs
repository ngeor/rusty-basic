use rusty_pc::map_err::{MapErrParser, SoftErrorOverrider};
use rusty_pc::{Parser, ParserErrorTrait, SurroundMode, surround};

use crate::error::ParserError;
use crate::input::StringView;
use crate::tokens::whitespace_ignoring;

pub trait OrExpected<C>: Parser<StringView, C, Error = ParserError>
where
    Self: Sized,
{
    /// Demands a successful result or returns a fatal syntax error
    /// with an error message like "Expected: " followed by the
    /// given expectation message.
    fn or_expected(self, expectation: &str) -> MapErrParser<Self, SoftErrorOverrider<Self::Error>> {
        self.or_fail(ParserError::expected(expectation).to_fatal())
    }
}

impl<P, C> OrExpected<C> for P where P: Parser<StringView, C, Error = ParserError> {}

pub trait PaddedByWs: Parser<StringView, Error = ParserError>
where
    Self: Sized,
{
    fn padded_by_ws(self) -> impl Parser<StringView, Output = Self::Output, Error = Self::Error> {
        surround(
            whitespace_ignoring(),
            self,
            whitespace_ignoring(),
            SurroundMode::Optional,
        )
    }
}

impl<P> PaddedByWs for P where P: Parser<StringView, Error = ParserError> {}
