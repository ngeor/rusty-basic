use crate::common::QErrorNode;
use crate::linter::converter::converter::ConverterImpl;
use crate::parser::{ExpressionNode, NameNode, Statement};

impl<'a> ConverterImpl<'a> {
    pub fn constant(
        &mut self,
        left: NameNode,
        right: ExpressionNode,
    ) -> Result<Statement, QErrorNode> {
        self.context
            .on_const(left, right)
            .map(|(left, right, calc)| Statement::Const(left, right, calc))
    }
}
