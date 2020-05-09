use super::error::*;
use super::post_conversion_linter::*;
use super::types::*;

pub struct NoDynamicConst;

impl NoDynamicConst {
    fn visit_const_expr(&self, e_node: &ExpressionNode) -> Result<(), Error> {
        let e: &Expression = e_node.as_ref();
        match e {
            Expression::FunctionCall(_, _) | Expression::Variable(_) => {
                err_l(LinterError::InvalidConstant, e_node)
            }
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
    fn visit_const(&self, _left_node: &QNameNode, e_node: &ExpressionNode) -> Result<(), Error> {
        self.visit_const_expr(e_node)
    }
}
