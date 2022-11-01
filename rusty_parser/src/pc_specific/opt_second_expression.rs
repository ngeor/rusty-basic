use crate::expression::ws_expr_node;
use crate::pc::{Parser, ParserOnce};
use crate::pc_specific::{keyword, whitespace};
use crate::types::Keyword;
use crate::{ExpressionNode, ExpressionTrait};
use rusty_common::*;

/// Finds the rightmost expression of a given type,
/// so that it can be determined if it ended in parenthesis or not.
pub trait ExtractExpression {
    fn to_expression(&self) -> &ExpressionNode;
}

impl ExtractExpression for ExpressionNode {
    fn to_expression(&self) -> &ExpressionNode {
        self
    }
}

pub fn opt_second_expression_after_keyword<P>(
    parser: P,
    keyword: Keyword,
) -> impl Parser<Output = (P::Output, Option<ExpressionNode>)>
where
    P: Parser,
    P::Output: ExtractExpression,
{
    parser.chain(move |first: P::Output| {
        let first_expr = first.to_expression();
        let is_paren = first_expr.is_parenthesis();
        parse_second(keyword, is_paren)
            .to_parser_once()
            .map(|opt_second| (first, opt_second))
    })
}

fn parse_second(k: Keyword, is_paren: bool) -> impl Parser<Output = Option<ExpressionNode>> {
    whitespace()
        .allow_none_if(is_paren)
        .and(keyword(k))
        .then_demand(ws_expr_node().or_fail(err(k)))
        .allow_none()
}

fn err(keyword: Keyword) -> QError {
    QError::SyntaxError(format!("Expected: expression after {}", keyword))
}