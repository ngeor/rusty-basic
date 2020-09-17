use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::common::*;

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_select_case(&self, s: &SelectCaseNode) -> Result<(), QErrorNode> {
        let top_qualifier: TypeDefinition = s.expr.type_definition();
        for c in s.case_blocks.iter() {
            match &c.expr {
                CaseExpression::Simple(expr) => {
                    let expr_qualifier = expr.type_definition();
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(expr);
                    }
                }
                CaseExpression::Range(from, to) => {
                    let expr_qualifier = from.type_definition();
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(from);
                    }

                    let expr_qualifier = to.type_definition();
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(to);
                    }
                }
                CaseExpression::Is(_, expr) => {
                    let expr_qualifier = expr.type_definition();
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(expr);
                    }
                }
            }
        }
        Ok(())
    }
}
