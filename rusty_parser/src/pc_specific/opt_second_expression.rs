use crate::expression::ws_expr_pos_p;
use crate::pc::{AllowNoneIf, And, AndWithoutUndo, Errors, Parser, RcStringView, ToOption};
use crate::pc_specific::{keyword, whitespace};
use crate::types::Keyword;
use crate::{ExpressionPos, ExpressionTrait, ParseError};

/// Finds the rightmost expression of a given type,
/// so that it can be determined if it ended in parenthesis or not.
pub trait ExtractExpression {
    fn to_expression(&self) -> &ExpressionPos;
}

impl ExtractExpression for ExpressionPos {
    fn to_expression(&self) -> &ExpressionPos {
        self
    }
}

pub fn opt_second_expression_after_keyword<P>(
    parser: P,
    keyword: Keyword,
) -> impl Parser<RcStringView, Output = (P::Output, Option<ExpressionPos>)>
where
    P: Parser<RcStringView>,
    P::Output: ExtractExpression,
{
    parser.chain(
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
    whitespace()
        .allow_none_if(is_paren)
        .and_tuple(keyword(k))
        .and_without_undo_keep_right(ws_expr_pos_p().or_fail(err(k)))
        .to_option()
}

fn err(keyword: Keyword) -> ParseError {
    ParseError::SyntaxError(format!("Expected: expression after {}", keyword))
}
