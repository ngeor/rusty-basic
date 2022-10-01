use super::post_conversion_linter::PostConversionLinter;
use crate::common::*;
use crate::parser::{CaseExpression, ExpressionNode};

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_case_expression(
        &mut self,
        case_expr: &CaseExpression,
        select_expr: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        match case_expr {
            CaseExpression::Simple(expr) => {
                if !expr.as_ref().can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(expr);
                }
            }
            CaseExpression::Range(from, to) => {
                if !from.as_ref().can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(from);
                }

                if !to.as_ref().can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(to);
                }
            }
            CaseExpression::Is(_, expr) => {
                if !expr.as_ref().can_cast_to(select_expr) {
                    return Err(QError::TypeMismatch).with_err_at(expr);
                }
            }
        }
        Ok(())
    }
}
