use crate::error::ParseError;
use crate::pc::boxed::boxed;
use crate::pc::{And, AndWithoutUndo, Chain, Errors, Map, Parser, RcStringView, ToOption, Token};
use crate::specific::core::expression::ws_expr_pos_p;
use crate::specific::pc_specific::{keyword, opt_whitespace, whitespace};
use crate::specific::{ExpressionPos, ExpressionTrait, Keyword};

/// Finds the rightmost expression of a given type,
/// so that it can be determined if it ended in parenthesis or not.
#[deprecated]
pub trait ExtractExpression {
    fn to_expression(&self) -> &ExpressionPos;
}

impl ExtractExpression for ExpressionPos {
    fn to_expression(&self) -> &ExpressionPos {
        self
    }
}

/// Parses an optional second expression that follows the first expression
/// and a keyword.
///
/// If the keyword is present, the second expression is mandatory.
///
/// Example: `FOR I = 1 TO 100 [STEP 5]`
pub fn opt_second_expression_after_keyword<P>(
    first_expression_parser: P,
    keyword: Keyword,
) -> impl Parser<RcStringView, Output = (P::Output, Option<ExpressionPos>)>
where
    P: Parser<RcStringView>,
    P::Output: ExtractExpression,
{
    first_expression_parser.chain(
        move |first| {
            let first_expr = first.to_expression();
            let is_paren = first_expr.is_parenthesis();
            parse_second(keyword, is_paren)
        },
        |first, opt_second| (first, opt_second),
    )
}

fn parse_second(
    k: Keyword,
    is_paren: bool,
) -> impl Parser<RcStringView, Output = Option<ExpressionPos>> {
    conditionally_opt_whitespace(is_paren)
        .and_tuple(keyword(k))
        .and_without_undo_keep_right(ws_expr_pos_p().or_fail(err(k)))
        .to_option()
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
