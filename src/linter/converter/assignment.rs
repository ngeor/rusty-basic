use crate::common::{AtLocation, Locatable};
use crate::linter::converter::{ConverterImpl, R};
use crate::parser::{ExpressionNode, Statement, StatementNode};

impl<'a> ConverterImpl<'a> {
    pub fn assignment(
        &mut self,
        name_expr_node: ExpressionNode,
        expression_node: ExpressionNode,
    ) -> R<StatementNode> {
        self.context
            .on_assignment(name_expr_node, expression_node)
            .map(|(Locatable { element: left, pos }, right, implicit_vars)| {
                (Statement::Assignment(left, right).at(pos), implicit_vars)
            })
    }
}
