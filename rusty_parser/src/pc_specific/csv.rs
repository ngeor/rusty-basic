use rusty_pc::*;

use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::SpecificTrait;
use crate::tokens::comma_ws;

/// Comma separated list of items.
pub fn csv<L: Parser<RcStringView, Error = ParseError>>(
    parser: L,
) -> impl Parser<RcStringView, Output = Vec<L::Output>, Error = ParseError> {
    delimited_by(parser, comma_ws(), trailing_comma_error())
}

pub fn csv_non_opt<P: Parser<RcStringView, Error = ParseError>>(
    parser: P,
    expectation: &str,
) -> impl Parser<RcStringView, Output = Vec<P::Output>, Error = ParseError> + use<'_, P> {
    csv(parser).or_expected(expectation)
}

pub fn trailing_comma_error() -> ParseError {
    ParseError::syntax_error("Error: trailing comma")
}
