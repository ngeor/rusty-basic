use super::post_conversion_linter::*;
use crate::common::*;
use crate::linter::types::*;
use crate::parser::TypeQualifier;

pub struct ForNextCounterMatch;

impl ForNextCounterMatch {
    fn ensure_numeric_variable(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        let var_type: ExpressionType = f.variable_name.dim_type().expression_type();
        match var_type {
            ExpressionType::BuiltIn(TypeQualifier::DollarString) => {
                Err(QError::TypeMismatch).with_err_no_pos()
            }
            ExpressionType::BuiltIn(_) => Ok(()),
            _ => Err(QError::TypeMismatch).with_err_no_pos(),
        }
    }

    fn ensure_for_next_counter_match(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        match &f.next_counter {
            Some(Locatable { element, pos }) => {
                if *element == f.variable_name {
                    Ok(())
                } else {
                    Err(QError::NextWithoutFor).with_err_at(*pos)
                }
            }
            None => Ok(()),
        }
    }
}

impl PostConversionLinter for ForNextCounterMatch {
    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&f.statements)?;
        self.ensure_numeric_variable(f)?;
        self.ensure_for_next_counter_match(f)
    }
}
