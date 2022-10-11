use super::expression_reducer::*;
use crate::common::*;
use crate::linter::pre_linter::HasFunctionView;
use crate::parser::Expression;

/// Finds undefined functions and converts them to zeroes.
pub struct UndefinedFunctionReducer<'a, R> {
    pub context: &'a R,
}

impl<'a, R> ExpressionReducer for UndefinedFunctionReducer<'a, R>
where
    R: HasFunctionView,
{
    fn visit_expression(&mut self, expression: Expression) -> Result<Expression, QErrorNode> {
        match expression {
            Expression::BinaryExpression(op, left, right, _) => {
                let mapped_left = self.visit_expression_node(*left)?;
                let mapped_right = self.visit_expression_node(*right)?;
                Expression::binary(mapped_left, mapped_right, op)
            }
            Expression::UnaryExpression(op, child) => {
                let mapped_child = self.visit_expression_node(*child)?;
                Ok(Expression::UnaryExpression(op, Box::new(mapped_child)))
            }
            Expression::FunctionCall(name, args) => {
                if self.context.functions().contains_key(name.bare_name()) {
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
