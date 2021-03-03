use super::post_conversion_linter::PostConversionLinter;
use crate::common::*;
use crate::parser::{CaseBlockNode, CaseExpression, ExpressionNode, SelectCaseNode};

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_select_case(&mut self, select_case_node: &SelectCaseNode) -> Result<(), QErrorNode> {
        for case_block_node in select_case_node.case_blocks.iter() {
            Self::visit_case_block_node(case_block_node, &select_case_node.expr)?;
        }
        Ok(())
    }
}

impl SelectCaseLinter {
    fn visit_case_block_node(
        case_block_node: &CaseBlockNode,
        select_expr: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        for case_expr in &case_block_node.expression_list {
            Self::visit_case_expression(case_expr, select_expr)?;
        }
        Ok(())
    }

    fn visit_case_expression(
        case_expr: &CaseExpression,
        select_expr: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        match case_expr {
            CaseExpression::Simple(expr) => {
                if !expr.can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(expr);
                }
            }
            CaseExpression::Range(from, to) => {
                if !from.can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(from);
                }

                if !to.can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(to);
                }
            }
            CaseExpression::Is(_, expr) => {
                if !expr.can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(expr);
                }
            }
        }
        Ok(())
    }
}
