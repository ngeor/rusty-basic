use crate::linter::converter::context::ExprContext;
use crate::linter::converter::{Converter, ConverterImpl, ConverterWithImplicitVariables, R};
use crate::parser::DoLoopNode;

impl<'a> ConverterWithImplicitVariables<DoLoopNode, DoLoopNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        do_loop_node: DoLoopNode,
    ) -> R<DoLoopNode> {
        let DoLoopNode {
            condition,
            statements,
            position,
            kind,
        } = do_loop_node;
        let (condition, implicit_vars) = self
            .context
            .on_expression(condition, ExprContext::Default)?;
        let statements = self.convert(statements)?;
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
