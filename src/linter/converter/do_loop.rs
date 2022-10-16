use crate::linter::converter::conversion_traits::SameTypeConverterWithImplicits;
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::DoLoopNode;

impl SameTypeConverterWithImplicits<DoLoopNode> for ConverterImpl {
    fn convert_same_type_with_implicits(&mut self, do_loop_node: DoLoopNode) -> R<DoLoopNode> {
        let DoLoopNode {
            condition,
            statements,
            position,
            kind,
        } = do_loop_node;
        let (condition, mut implicit_vars) = self
            .context
            .on_expression(condition, ExprContext::Default)?;
        let (statements, mut block_implicits) = self.convert_block_keeping_implicits(statements)?;
        implicit_vars.append(&mut block_implicits);
        Ok((
            DoLoopNode {
                condition,
                statements,
                position,
                kind,
            },
            implicit_vars,
        ))
    }
}
