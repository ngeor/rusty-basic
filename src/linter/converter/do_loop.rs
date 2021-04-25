use crate::linter::converter::conversion_traits::{
    SameTypeConverter, SameTypeConverterWithImplicits,
};
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::DoLoopNode;

impl<'a> SameTypeConverterWithImplicits<DoLoopNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, do_loop_node: DoLoopNode) -> R<DoLoopNode> {
        let DoLoopNode {
            condition,
            statements,
            position,
            kind,
        } = do_loop_node;
        let (condition, implicit_vars) = self
            .context
            .on_expression(condition, ExprContext::Default)?;
        let statements = self.convert_same_type(statements)?;
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
