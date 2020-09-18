use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::common::*;

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_select_case(&self, select_case_node: &SelectCaseNode) -> Result<(), QErrorNode> {
        for c in select_case_node.case_blocks.iter() {
            match &c.expr {
                CaseExpression::Simple(expr) => {
                    if !expr.can_cast_to(&select_case_node.expr) {
                        return Err(QError::TypeMismatch).with_err_at(expr);
                    }
                }
                CaseExpression::Range(from, to) => {
                    if !from.can_cast_to(&select_case_node.expr) {
                        return Err(QError::TypeMismatch).with_err_at(from);
                    }

                    if !to.can_cast_to(&select_case_node.expr) {
                        return Err(QError::TypeMismatch).with_err_at(to);
                    }
                }
                CaseExpression::Is(_, expr) => {
                    if !expr.can_cast_to(&select_case_node.expr) {
                        return Err(QError::TypeMismatch).with_err_at(expr);
                    }
                }
            }
        }
        Ok(())
    }
}
