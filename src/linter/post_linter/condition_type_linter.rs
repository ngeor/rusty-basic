use crate::common::{QError, QErrorNode, ToLocatableError};
use crate::linter::post_linter::post_conversion_linter::PostConversionLinter;
use crate::parser::{
    ConditionalBlockNode, DoLoopNode, ExpressionNode, ExpressionType, HasExpressionType,
    TypeQualifier,
};

/// Ensures that expressions appearing in logical conditions are numeric.
pub struct ConditionTypeLinter {}

impl ConditionTypeLinter {
    fn ensure_expression_is_condition(expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
        match expr_node.as_ref().expression_type() {
            ExpressionType::BuiltIn(q) => {
                if q == TypeQualifier::DollarString {
                    Err(QError::TypeMismatch).with_err_at(expr_node)
                } else {
                    Ok(())
                }
            }
            _ => Err(QError::TypeMismatch).with_err_at(expr_node),
        }
    }
}

impl PostConversionLinter for ConditionTypeLinter {
    fn visit_conditional_block(&mut self, c: &ConditionalBlockNode) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&c.statements)?;
        Self::ensure_expression_is_condition(&c.condition)
    }

    fn visit_do_loop(&mut self, do_loop_node: &DoLoopNode) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&do_loop_node.statements)?;
        Self::ensure_expression_is_condition(&do_loop_node.condition)
    }
}
