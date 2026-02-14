use rusty_pc::and::TupleCombiner;
use rusty_pc::{IifParser, Parser, ParserErrorTrait};

use crate::error::ParserError;
use crate::expr::parsers::ws_expr_pos_p;
use crate::input::StringView;
use crate::pc_specific::{keyword, whitespace_ignoring};
use crate::{ExpressionPos, Keyword};

/// Parses an optional second expression that follows the first expression
/// and a keyword.
///
/// If the keyword is present, the second expression is mandatory.
///
/// Example: `FOR I = 1 TO 100 [STEP 5]`
pub fn opt_second_expression_after_keyword<P, F>(
    first_parser: P,
    keyword: Keyword,
    is_first_wrapped_in_parenthesis: F,
) -> impl Parser<StringView, Output = (P::Output, Option<ExpressionPos>), Error = ParserError>
where
    P: Parser<StringView, Error = ParserError>,
    F: Fn(&P::Output) -> bool + 'static,
{
    first_parser.then_with_in_context(
        parse_second(keyword),
        move |first| is_first_wrapped_in_parenthesis(first),
        TupleCombiner,
    )
}

// first_parser AND [ cond_ws(is_first_paren) KEYWORD !AND! ws_expr ]
fn parse_second(
    k: Keyword,
) -> impl Parser<StringView, bool, Output = Option<ExpressionPos>, Error = ParserError> {
    // the left side needs the context
    ws_keyword(k)
        .and_keep_right(
            // but the right side does not need it...
            ws_expr_pos_p().no_context().or_fail(err(k)),
        )
        // finally to_option needs to send the context down to the "and_keep_right"
        .to_option()
}

fn ws_keyword(k: Keyword) -> impl Parser<StringView, bool, Error = ParserError> {
    // the left side has the context
    conditionally_opt_whitespace().and_tuple(
        // but the right side does not
        keyword(k).no_context(),
    )
}

fn err(keyword: Keyword) -> ParserError {
    ParserError::expected(&format!("expression after {}", keyword)).to_fatal()
}

/// Creates a parser that parses whitespace,
/// conditionally allowing it to be missing.
/// When [allow_none] is false, whitespace is mandatory.
/// When [allow_none] is true, the whitespace can be missing.
/// This is typically the case when the previously parsed
/// token was a right side parenthesis.
///
/// Examples
///
/// * `(1 + 2)AND` no whitespace is required before `AND`
/// * `1 + 2AND` the lack of whitespace before `AND` is an error
pub(super) fn conditionally_opt_whitespace()
-> impl Parser<StringView, bool, Output = (), Error = ParserError> {
    IifParser::new(
        // allow none
        whitespace_ignoring().to_option().map_to_unit(),
        // whitespace is required
        whitespace_ignoring(),
    )
}
