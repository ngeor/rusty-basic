use super::built_in_function_linter::is_built_in_function;
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
                if is_built_in_function(name.bare_name())
                    || self.functions.contains_key(name.bare_name())
                {
                    let r_args: Vec<ExpressionNode> = args
                        .into_iter()
                        .map(|a| self.visit_expression_node(a))
                        .collect::<Result<Vec<ExpressionNode>, Error>>()?;
                    Ok(Expression::FunctionCall(name, r_args))
                } else {
                    // the user_defined_function_linter already ensures that the args are valid
                    Ok(Expression::IntegerLiteral(0))
                }
            }
            _ => Ok(expression),
        }
    }
}
