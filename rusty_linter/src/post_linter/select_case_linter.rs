use super::post_conversion_linter::PostConversionLinter;
use crate::error::{LintError, LintErrorPos};
use crate::CanCastTo;
use rusty_common::*;
use rusty_parser::{CaseExpression, ExpressionPos};

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_case_expression(
        &mut self,
        case_expr: &CaseExpression,
        select_expr: &ExpressionPos,
    ) -> Result<(), LintErrorPos> {
        match case_expr {
            CaseExpression::Simple(expr) => {
                if !expr.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch).with_err_at(expr);
                }
            }
            CaseExpression::Range(from, to) => {
                if !from.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch).with_err_at(from);
                }

                if !to.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch).with_err_at(to);
                }
            }
            CaseExpression::Is(_, expr) => {
                if !expr.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch).with_err_at(expr);
                }
            }
        }
        Ok(())
    }
}
