use rusty_pc::{
    And, FlatMapOkNone, Flatten, Map, NoContext, OrFail, Parser, SetContext, ThenWithContext, ToOption, Token, ctx_parser
};

use crate::core::expression::ws_expr_pos_p;
use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::keyword;
use crate::tokens::whitespace;
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
) -> impl Parser<RcStringView, Output = (P::Output, Option<ExpressionPos>), Error = ParseError>
where
    P: Parser<RcStringView, Error = ParseError>,
    F: Fn(&P::Output) -> bool + 'static,
{
    first_parser.then_with_in_context(
        move |first| is_first_wrapped_in_parenthesis(first),
        move || parse_second(keyword),
    )
}

// first_parser AND [ cond_ws(is_first_paren) KEYWORD !AND! ws_expr ]
fn parse_second(
    k: Keyword,
) -> impl Parser<RcStringView, bool, Output = Option<ExpressionPos>, Error = ParseError> + SetContext<bool>
{
    // the left side needs the context
    ws_keyword(k)
        .and_keep_right(
            // but the right side does not need it...
            ws_expr_pos_p().no_context().or_fail(err(k)),
        )
        // finally to_option needs to send the context down to the "and_keep_right"
        .to_option()
}

fn ws_keyword(
    k: Keyword,
) -> impl Parser<RcStringView, bool, Error = ParseError> + SetContext<bool> {
    // the left side has the context
    conditionally_opt_whitespace().and_tuple(
        // but the right side does not
        keyword(k).no_context(),
    )
}

fn err(keyword: Keyword) -> ParseError {
    ParseError::SyntaxError(format!("Expected: expression after {}", keyword))
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
-> impl Parser<RcStringView, bool, Output = Option<Token>, Error = ParseError> + SetContext<bool> {
    ctx_parser()
        .map(|allow_none| {
            whitespace()
                .flat_map_ok_none(
                    |i, ok| Ok((i, Some(ok))),
                    move |i| {
                        if allow_none {
                            Ok((i, None))
                        } else {
                            Err((false, i, ParseError::default()))
                        }
                    },
                )
                .no_context()
        })
        .flatten()
}
