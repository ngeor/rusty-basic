use super::post_conversion_linter::PostConversionLinter;
use crate::core::CanCastTo;
use crate::core::{LintError, LintErrorPos};
use rusty_common::AtPos;
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
                    return Err(LintError::TypeMismatch.at(expr));
                }
            }
            CaseExpression::Range(from, to) => {
                if !from.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch.at(from));
                }

                if !to.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch.at(to));
                }
            }
            CaseExpression::Is(_, expr) => {
                if !expr.can_cast_to(select_expr) {
                    return Err(LintError::TypeMismatch.at(expr));
                }
            }
        }
        Ok(())
    }
}
