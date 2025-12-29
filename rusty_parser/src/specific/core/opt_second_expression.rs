use crate::error::ParseError;
use crate::input::RcStringView;
use crate::pc::boxed::boxed;
use crate::pc::{And, Errors, Map, Parser, ThenWith, ToOption, Token};
use crate::specific::core::expression::ws_expr_pos_p;
use crate::specific::pc_specific::{keyword, opt_whitespace, whitespace};
use crate::specific::{ExpressionPos, Keyword};

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
) -> impl Parser<RcStringView, Output = (P::Output, Option<ExpressionPos>)>
where
    P: Parser<RcStringView>,
    F: Fn(&P::Output) -> bool,
{
    first_parser.then_with(
        move |first| {
            let is_paren = is_first_wrapped_in_parenthesis(&first);
            parse_second(keyword, is_paren).to_option()
        },
        |first, opt_second| (first, opt_second),
    )
}

// first_parser AND [ cond_ws(is_first_paren) KEYWORD !AND! ws_expr ]
fn parse_second(
    k: Keyword,
    is_preceded_by_paren: bool,
) -> impl Parser<RcStringView, Output = ExpressionPos> {
    ws_keyword(k, is_preceded_by_paren).and_keep_right(ws_expr_pos_p().or_fail(err(k)))
}

fn ws_keyword(k: Keyword, is_preceded_by_paren: bool) -> impl Parser<RcStringView> {
    conditionally_opt_whitespace(is_preceded_by_paren).and_tuple(keyword(k))
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
pub(super) fn conditionally_opt_whitespace(
    allow_none: bool,
) -> impl Parser<RcStringView, Output = Option<Token>> {
    if allow_none {
        boxed(opt_whitespace())
    } else {
        boxed(whitespace().map(Some))
    }
}
