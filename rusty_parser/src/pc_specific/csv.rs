use crate::error::ParseError;
use crate::pc::*;
use crate::pc_specific::*;

/// Comma separated list of items.
pub fn csv<L: Parser<RcStringView>>(
    parser: L,
) -> impl Parser<RcStringView, Output = Vec<L::Output>> {
    delimited_by(parser, comma(), trailing_comma_error())
}

pub fn csv_non_opt<P: Parser<RcStringView>>(
    parser: P,
    err: &str,
) -> impl Parser<RcStringView, Output = Vec<P::Output>> + use<'_, P> {
    csv(parser).or_syntax_error(err)
}

pub fn trailing_comma_error() -> ParseError {
    ParseError::syntax_error("Error: trailing comma")
}
