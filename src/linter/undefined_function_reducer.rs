use super::error::*;
use super::expression_reducer::*;
use super::subprogram_context::FunctionMap;
use super::types::*;
use crate::parser::NameTrait;

pub struct UndefinedFunctionReducer<'a> {
    pub functions: &'a FunctionMap,
}

impl<'a> ExpressionReducer for UndefinedFunctionReducer<'a> {
    fn visit_expression(&self, expression: Expression) -> Result<Expression, Error> {
        match expression {
            Expression::BinaryExpression(op, left, right) => {
                let mapped_left = self.visit_expression_node(*left)?;
                let mapped_right = self.visit_expression_node(*right)?;
                Ok(Expression::BinaryExpression(
                    op,
                    Box::new(mapped_left),
                    Box::new(mapped_right),
                ))
            }
            Expression::UnaryExpression(op, child) => {
                let mapped_child = self.visit_expression_node(*child)?;
                Ok(Expression::UnaryExpression(op, Box::new(mapped_child)))
            }
            Expression::FunctionCall(name, args) => {
                if self.functions.contains_key(name.bare_name()) {
                    Ok(Expression::FunctionCall(
                        name,
                        self.visit_expression_nodes(args)?,
                    ))
                } else {
                    // the user_defined_function_linter already ensures that the args are valid
                    Ok(Expression::IntegerLiteral(0))
                }
            }
            Expression::BuiltInFunctionCall(name, args) => Ok(Expression::BuiltInFunctionCall(
                name,
                self.visit_expression_nodes(args)?,
            )),
            _ => Ok(expression),
        }
    }
}
