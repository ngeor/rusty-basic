use crate::core::*;
use rusty_common::*;
use rusty_parser::*;

pub struct ForNextCounterMatch;

no_op_visitor!(ForNextCounterMatch: DefType, FunctionDeclaration, FunctionImplementation, SubDeclaration, SubImplementation, UserDefinedType);
no_pos_visitor!(ForNextCounterMatch);

impl ForNextCounterMatch {
    pub fn visitor() -> impl Visitor<Program> + SetPosition {
        DeepStatementVisitor::new(Self)
    }
}

impl Visitor<Statement> for ForNextCounterMatch {
    fn visit(&mut self, element: &Statement) -> VisitResult {
        match element {
            Statement::ForLoop(f) => self.visit(f),
            _ => Ok(()),
        }
    }
}

impl Visitor<ForLoop> for ForNextCounterMatch {
    fn visit(&mut self, f: &ForLoop) -> crate::core::VisitResult {
        self.ensure_numeric_variable(f)?;
        self.ensure_for_next_counter_match(f)
    }
}

impl ForNextCounterMatch {
    fn ensure_numeric_variable(&self, f: &ForLoop) -> Result<(), LintErrorPos> {
        let Positioned {
            element: var_expr,
            pos,
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
                    Err(LintError::TypeMismatch.at_pos(*pos))
                }
                ExpressionType::BuiltIn(_) => Ok(()),
                _ => Err(LintError::TypeMismatch.at_pos(*pos)),
            },
            _ => panic!("It should not be possible for the FOR variable to be something othe than a variable"),
        }
    }

    fn ensure_for_next_counter_match(&self, f: &ForLoop) -> Result<(), LintErrorPos> {
        let Positioned {
            element: var_expr, ..
        } = &f.variable_name;
        if let Some(Positioned {
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
                            Err(LintError::NextWithoutFor.at(pos))
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
