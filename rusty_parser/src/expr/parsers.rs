use rusty_pc::and::{KeepLeftCombiner, TupleCombiner, VecCombiner};
use rusty_pc::*;

use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{ExpressionPos, ExpressionTrait, Expressions, Keyword, ParserError};

/// Parses an expression.
pub fn expression_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    lazy(super::binary_expression::parser)
}

/// `( expr [, expr]* )`
pub fn in_parenthesis_csv_expressions_non_opt(
    expectation: &str,
) -> impl Parser<StringView, Output = Expressions, Error = ParserError> + '_ {
    in_parenthesis(csv_expressions_non_opt(expectation)).to_fatal()
}

/// Parses one or more expressions separated by comma.
/// FIXME Unlike csv_expressions, the first expression does not need a separator!
pub fn csv_expressions_non_opt(
    expectation: &str,
) -> impl Parser<StringView, Output = Expressions, Error = ParserError> + use<'_> {
    csv_non_opt(expression_pos_p(), expectation)
}

/// Parses one or more expressions separated by comma.
/// Trailing commas are not allowed.
/// Missing expressions are not allowed.
/// The first expression needs to be preceded by space or surrounded in parenthesis.
pub fn csv_expressions_first_guarded()
-> impl Parser<StringView, Output = Expressions, Error = ParserError> {
    ws_expr_pos_p().map(|first| vec![first]).and(
        comma_ws()
            .and_keep_right(expression_pos_p().or_expected("expression after comma"))
            .zero_or_more(),
        VecCombiner,
    )
}

/// Parses an expression that is either preceded by whitespace
/// or is a parenthesis expression.
///
/// ```text
/// <expr-in-parenthesis> |
/// <ws> <expr>
/// ```
pub fn ws_expr_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    super::parenthesis::parser().or(lead_ws(expression_pos_p()))
}

/// Parses an expression that is either surrounded by whitespace
/// or is a parenthesis expression.
///
/// The whitespace is mandatory after a non-parenthesis
/// expression.
///
/// ```text
/// <ws> <expr-not-in-parenthesis> <ws> |
/// <ws> <expr-in-parenthesis> <ws> |
/// <expr-in-parenthesis>
/// ```
pub fn ws_expr_pos_ws_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    followed_by_ws(ws_expr_pos_p())
}

fn followed_by_ws(
    parser: impl Parser<StringView, Output = ExpressionPos, Error = ParserError>,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    parser.then_with_in_context(
        conditionally_opt_whitespace(),
        |e| e.is_parenthesis(),
        KeepLeftCombiner,
    )
}

/// Parses an expression,
/// then demands whitespace, unless the expression is a parenthesis.
/// Finally it demands the given keyword.
pub fn expr_ws_keyword_p(
    keyword: Keyword,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    expr_ws_followed_by(expression_pos_p(), keyword_ignoring(keyword))
}

/// Parses an expression, failing fatally with the given expectation message if it can't be parsed.
/// Then it demands whitespace, unless the expression is a parenthesis.
/// Finally it demands the given keyword.
pub fn demand_expr_ws_keyword_p(
    expectation: &str,
    keyword: Keyword,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    expr_ws_followed_by(
        expression_pos_p().or_expected(expectation),
        keyword_ignoring(keyword),
    )
}

pub fn ws_expr_ws_keyword_p(
    keyword: Keyword,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    expr_ws_followed_by(ws_expr_pos_p(), keyword_ignoring(keyword))
}

pub fn demand_ws_expr_ws_keyword_p(
    expectation: &str,
    keyword: Keyword,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    expr_ws_followed_by(
        ws_expr_pos_p().or_expected(expectation),
        keyword_ignoring(keyword),
    )
}

/// Parses the expression using the given parser,
/// then demands whitespace, unless the expression is a parenthesis.
/// Finally it demands the second parser.
pub fn expr_ws_followed_by(
    expr_parser: impl Parser<StringView, Output = ExpressionPos, Error = ParserError>,
    other_parser: impl Parser<StringView, Error = ParserError>,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    expr_parser.then_with_in_context(
        conditionally_opt_whitespace()
            .to_fatal()
            .and_keep_right(other_parser.to_fatal().no_context()),
        |e| e.is_parenthesis(),
        KeepLeftCombiner,
    )
}

/// Parses an expression, followed optionally by a keyword and a second expression.
/// If the keyword is present, the second expression is mandatory.
///
/// Examples: `FOR I = 1 TO 100 [STEP 5]`, `CASE 1 [TO 2]`
pub fn expr_keyword_opt_expr(
    keyword: Keyword,
) -> impl Parser<StringView, Output = (ExpressionPos, Option<ExpressionPos>), Error = ParserError> {
    expression_pos_p().then_with_in_context(
        opt_keyword_expr(keyword),
        ExpressionTrait::is_parenthesis,
        TupleCombiner,
    )
}

/// Parses the optional `TO expr` part (e.g. `CASE 1 TO 2`)
fn opt_keyword_expr(
    keyword: Keyword,
) -> impl Parser<StringView, bool, Output = Option<ExpressionPos>, Error = ParserError> {
    let msg = format!("expression after {}", keyword);
    conditionally_opt_whitespace()
        .and_keep_right(keyword_ignoring(keyword).no_context())
        .and_keep_right(ws_expr_pos_p().or_expected(&msg).no_context())
        .to_option()
}
