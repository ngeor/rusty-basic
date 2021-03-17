use crate::built_ins::{linter, BuiltInSub};
use crate::common::*;
use crate::parser::{Expression, ExpressionNode};

use super::post_conversion_linter::PostConversionLinter;

/// Lints built-in functions and subs.
pub struct BuiltInLinter;

impl PostConversionLinter for BuiltInLinter {
    fn visit_built_in_sub_call(
        &mut self,
        built_in_sub: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)?;
        linter::lint_sub_call(built_in_sub, args)
    }

    fn visit_expression(&mut self, expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
        let pos = expr_node.pos();
        let e = expr_node.as_ref();
        match e {
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                linter::lint_function_call(built_in_function, args).patch_err_pos(pos)
            }
            Expression::BinaryExpression(_, left, right, _) => {
                self.visit_expression(left)?;
                self.visit_expression(right)
            }
            Expression::UnaryExpression(_, child) => self.visit_expression(child),
            _ => Ok(()),
        }
    }
}
