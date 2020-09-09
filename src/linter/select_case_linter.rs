use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::common::*;
use crate::parser::CanCastTo;

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_select_case(&self, s: &SelectCaseNode) -> Result<(), QErrorNode> {
        let top_qualifier: ResolvedTypeDefinition = s.expr.try_type_definition()?;
        for c in s.case_blocks.iter() {
            match &c.expr {
                CaseExpression::Simple(expr) => {
                    let expr_qualifier = expr.try_type_definition()?;
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(expr);
                    }
                }
                CaseExpression::Range(from, to) => {
                    let expr_qualifier = from.try_type_definition()?;
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(from);
                    }

                    let expr_qualifier = to.try_type_definition()?;
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(to);
                    }
                }
                CaseExpression::Is(_, expr) => {
                    let expr_qualifier = expr.try_type_definition()?;
                    if !expr_qualifier.can_cast_to(&top_qualifier) {
                        return Err(QError::TypeMismatch).with_err_at(expr);
                    }
                }
            }
        }
        Ok(())
    }
}
