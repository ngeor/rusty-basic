use crate::common::*;
use crate::parser::{Expression, ExpressionType, ForLoopNode, TypeQualifier, VariableInfo};

use super::post_conversion_linter::*;

pub struct ForNextCounterMatch;

impl ForNextCounterMatch {
    fn ensure_numeric_variable(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        let Locatable {
            element: var_expr, ..
        } = &f.variable_name;
        match var_expr {
            Expression::Variable(
                _,
                VariableInfo {
                    expression_type: var_type,
                    ..
                },
            ) => match var_type {
                ExpressionType::BuiltIn(TypeQualifier::DollarString) => {
                    Err(QError::TypeMismatch).with_err_no_pos()
                }
                ExpressionType::BuiltIn(_) => Ok(()),
                _ => Err(QError::TypeMismatch).with_err_no_pos(),
            },
            _ => unimplemented!(),
        }
    }

    fn ensure_for_next_counter_match(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        let Locatable {
            element: var_expr, ..
        } = &f.variable_name;
        if let Some(Locatable {
            element: next_var_expr,
            pos,
        }) = &f.next_counter
        {
            match var_expr {
                Expression::Variable(var_name, _) => match next_var_expr {
                    Expression::Variable(next_var_name, _) => {
                        if var_name == next_var_name {
                            Ok(())
                        } else {
                            Err(QError::NextWithoutFor).with_err_at(*pos)
                        }
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        } else {
            // does not have a NEXT variable
            Ok(())
        }
    }
}

impl PostConversionLinter for ForNextCounterMatch {
    fn visit_for_loop(&mut self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&f.statements)?;
        self.ensure_numeric_variable(f)?;
        self.ensure_for_next_counter_match(f)
    }
}
