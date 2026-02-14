use rusty_pc::and::{KeepLeftCombiner, VecCombiner};
use rusty_pc::*;

use crate::expr::opt_second_expression::conditionally_opt_whitespace;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{ExpressionPos, ExpressionTrait, Expressions, ParserError};
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

pub fn expression_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    lazy(eager_expression_pos_p)
}

/// Parses an expression that is either preceded by whitespace
/// or is a parenthesis expression.
///
/// ```text
/// <ws> <expr-not-in-parenthesis> |
/// <ws> <expr-in-parenthesis> |
/// <expr-in-parenthesis>
/// ```
pub fn ws_expr_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    // ws* ( expr )
    // ws+ expr
    preceded_by_ws(expression_pos_p())
}

/// Parses an expression that is either followed by whitespace
/// or is a parenthesis expression.
///
/// The whitespace is mandatory after a non-parenthesis
/// expression.
///
/// ```text
/// <expr-not-in-parenthesis> <ws> |
/// <expr-in-parenthesis> <ws> |
/// <expr-in-parenthesis>
/// ```
pub fn expr_pos_ws_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    followed_by_ws(expression_pos_p())
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

fn preceded_by_ws(
    parser: impl Parser<StringView, Output = ExpressionPos, Error = ParserError>,
) -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    super::guard::parser().and_keep_right(parser)
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

/// Parses an expression
fn eager_expression_pos_p() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError>
{
    super::binary_expression::parser()
}
