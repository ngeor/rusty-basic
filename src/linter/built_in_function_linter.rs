use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::built_ins::BuiltInLint;
use crate::common::*;

pub struct BuiltInFunctionLinter;

impl PostConversionLinter for BuiltInFunctionLinter {
    fn visit_expression(&self, expr_node: &ExpressionNode) -> Result<(), Error> {
        let pos = expr_node.location();
        let e = expr_node.as_ref();
        match e {
            Expression::BuiltInFunctionCall(n, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                n.lint(args).with_err_pos(pos)
            }
            Expression::BinaryExpression(_, left, right) => {
                self.visit_expression(left)?;
                self.visit_expression(right)
            }
            Expression::UnaryExpression(_, child) => self.visit_expression(child),
            _ => Ok(()),
        }
    }
}
