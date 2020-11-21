use crate::common::QErrorNode;
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::parser::{ExpressionNode, QualifiedNameNode};

// Convert expression into an expression + a collection of implicitly declared variables

impl<'a> ConverterWithImplicitVariables<ExpressionNode, ExpressionNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        expression_node: ExpressionNode,
    ) -> Result<(ExpressionNode, Vec<QualifiedNameNode>), QErrorNode> {
        self.context.on_expression(expression_node)
    }
}
