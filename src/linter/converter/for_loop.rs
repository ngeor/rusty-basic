use crate::common::QErrorNode;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::ForLoopNode;

impl SameTypeConverter<ForLoopNode> for ConverterImpl {
    fn convert(&mut self, a: ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
        let variable_name = self
            .context
            .on_expression(a.variable_name, ExprContext::Assignment)?;
        let lower_bound = self
            .context
            .on_expression(a.lower_bound, ExprContext::Default)?;
        let upper_bound = self
            .context
            .on_expression(a.upper_bound, ExprContext::Default)?;
        let step = self
            .context
            .on_opt_expression(a.step, ExprContext::Default)?;
        let next_counter = self
            .context
            .on_opt_expression(a.next_counter, ExprContext::Assignment)?;
        let statements = self.convert_block_removing_constants(a.statements)?;
        Ok(ForLoopNode {
            variable_name,
            lower_bound,
            upper_bound,
            step,
            statements,
            next_counter,
        })
    }
}
