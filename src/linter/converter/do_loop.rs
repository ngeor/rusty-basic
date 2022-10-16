use crate::common::QErrorNode;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::DoLoopNode;

impl SameTypeConverter<DoLoopNode> for ConverterImpl {
    fn convert(&mut self, do_loop_node: DoLoopNode) -> Result<DoLoopNode, QErrorNode> {
        let DoLoopNode {
            condition,
            statements,
            position,
            kind,
        } = do_loop_node;
        let condition = self
            .context
            .on_expression(condition, ExprContext::Default)?;
        let statements = self.convert_block_removing_constants(statements)?;
        Ok(DoLoopNode {
            condition,
            statements,
            position,
            kind,
        })
    }
}
