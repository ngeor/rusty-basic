use rusty_pc::{Parser, SurroundMode, surround};

use crate::ParserError;
use crate::input::StringView;
use crate::tokens::{TokenType, any_token_of};

/// Parses a whitespace token dismissing it.
pub fn whitespace_ignoring() -> impl Parser<StringView, Output = (), Error = ParserError> {
    any_token_of!(TokenType::Whitespace).map_to_unit()
}

/// Parses optional leading and trailing whitespace around the given parser.
pub fn padded_by_ws<P>(
    parser: P,
) -> impl Parser<StringView, Output = P::Output, Error = ParserError>
where
    P: Parser<StringView, Error = ParserError>,
{
    surround(
        whitespace_ignoring(),
        parser,
        whitespace_ignoring(),
        SurroundMode::Optional,
    )
}
