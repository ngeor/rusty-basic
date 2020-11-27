use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::ConverterImpl;
use crate::parser::{ExpressionNode, QualifiedNameNode, Statement, StatementNode};

impl<'a> ConverterImpl<'a> {
    pub fn assignment(
        &mut self,
        name_expr_node: ExpressionNode,
        expression_node: ExpressionNode,
    ) -> Result<(StatementNode, Vec<QualifiedNameNode>), QErrorNode> {
        self.context
            .on_assignment(name_expr_node, expression_node)
            .map(|(Locatable { element: left, pos }, right, implicit_vars)| {
                (Statement::Assignment(left, right).at(pos), implicit_vars)
            })
    }
}
