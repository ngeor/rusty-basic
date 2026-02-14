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

/// Parses optional whitespace, dismissing the token.
/// This parser always succeeds.
pub fn opt_ws() -> impl Parser<StringView, Output = (), Error = ParserError> {
    whitespace_ignoring().to_option().map_to_unit()
}

/// Parses optional leading whitespace before the given parser.
pub fn lead_opt_ws<P>(parser: P) -> impl Parser<StringView, Output = P::Output, Error = ParserError>
where
    P: Parser<StringView, Error = ParserError>,
{
    opt_ws().and_keep_right(parser)
}

/// Parses leading whitespace before the given parser.
pub fn lead_ws<P>(parser: P) -> impl Parser<StringView, Output = P::Output, Error = ParserError>
where
    P: Parser<StringView, Error = ParserError>,
{
    whitespace_ignoring().and_keep_right(parser)
}

/// Parses mandatory whitespace.
pub fn demand_ws() -> impl Parser<StringView, Output = (), Error = ParserError> {
    whitespace_ignoring().to_fatal()
}

/// Parses mandatory leading whitespace before the given parser.
pub fn demand_lead_ws<P>(
    parser: P,
) -> impl Parser<StringView, Output = P::Output, Error = ParserError>
where
    P: Parser<StringView, Error = ParserError>,
{
    demand_ws().and_keep_right(parser)
}

pub fn demand_lead_ws_ctx<P, C>(
    parser: P,
) -> impl Parser<StringView, C, Output = P::Output, Error = ParserError>
where
    P: Parser<StringView, C, Error = ParserError>,
    C: Clone,
{
    demand_ws().no_context().and_keep_right(parser)
}
