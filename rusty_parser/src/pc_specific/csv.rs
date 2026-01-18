use rusty_pc::*;

use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::OrExpected;
use crate::tokens::comma_ws;

/// Comma separated list of items.
pub fn csv<L: Parser<StringView, Error = ParserError>>(
    parser: L,
) -> impl Parser<StringView, Output = Vec<L::Output>, Error = ParserError> {
    parser.delimited_by(comma_ws(), trailing_comma_error())
}

pub fn csv_non_opt<P: Parser<StringView, Error = ParserError>>(
    parser: P,
    expectation: &str,
) -> impl Parser<StringView, Output = Vec<P::Output>, Error = ParserError> + use<'_, P> {
    csv(parser).or_expected(expectation)
}

pub fn trailing_comma_error() -> ParserError {
    ParserError::syntax_error("Error: trailing comma")
}
