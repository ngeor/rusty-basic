use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::parser::TypeQualifier;

pub struct SelectCaseLinter;

impl PostConversionLinter for SelectCaseLinter {
    fn visit_select_case(&self, s: &SelectCaseNode) -> Result<(), Error> {
        let top_qualifier: TypeQualifier = s.expr.try_qualifier()?;
        for c in s.case_blocks.iter() {
            match &c.expr {
                CaseExpression::Simple(expr) => {
                    let expr_qualifier = expr.try_qualifier()?;
                    if !expr_qualifier.can_cast_to(top_qualifier) {
                        return err_l(LinterError::TypeMismatch, expr);
                    }
                }
                CaseExpression::Range(from, to) => {
                    let expr_qualifier = from.try_qualifier()?;
                    if !expr_qualifier.can_cast_to(top_qualifier) {
                        return err_l(LinterError::TypeMismatch, from);
                    }

                    let expr_qualifier = to.try_qualifier()?;
                    if !expr_qualifier.can_cast_to(top_qualifier) {
                        return err_l(LinterError::TypeMismatch, to);
                    }
                }
                CaseExpression::Is(_, expr) => {
                    let expr_qualifier = expr.try_qualifier()?;
                    if !expr_qualifier.can_cast_to(top_qualifier) {
                        return err_l(LinterError::TypeMismatch, expr);
                    }
                }
            }
        }
        Ok(())
    }
}
