use super::post_conversion_linter::*;
use super::types::*;
use crate::common::*;

pub struct NoDynamicConst;

impl NoDynamicConst {
    fn visit_const_expr(&self, e_node: &ExpressionNode) -> Result<(), QErrorNode> {
        let e: &Expression = e_node.as_ref();
        match e {
            Expression::FunctionCall(_, _)
            | Expression::BuiltInFunctionCall(_, _)
            | Expression::Variable(_) => Err(QError::InvalidConstant).with_err_at(e_node),
            Expression::BinaryExpression(_, left, right) => {
                let unboxed_left: &ExpressionNode = left;
                let unboxed_right: &ExpressionNode = right;
                self.visit_const_expr(unboxed_left)?;
                self.visit_const_expr(unboxed_right)
            }
            Expression::UnaryExpression(_, child) => {
                let unboxed_child: &ExpressionNode = child;
                self.visit_const_expr(unboxed_child)
            }
            _ => Ok(()),
        }
    }
}

impl PostConversionLinter for NoDynamicConst {
    fn visit_const(
        &self,
        _left_node: &QNameNode,
        e_node: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        self.visit_const_expr(e_node)
    }
}
