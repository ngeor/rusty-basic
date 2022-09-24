use crate::linter::converter::conversion_traits::SameTypeConverterWithImplicits;
use crate::linter::converter::{ConverterImpl, ExprContext, Implicits, R};
use crate::parser::ForLoopNode;

impl<'a> SameTypeConverterWithImplicits<ForLoopNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: ForLoopNode) -> R<ForLoopNode> {
        let mut implicits: Implicits = vec![];
        let (variable_name, mut implicit_variables_variable_name) = self
            .context
            .on_expression(a.variable_name, ExprContext::Assignment)?;
        implicits.append(&mut implicit_variables_variable_name);
        let (lower_bound, mut implicit_variables_lower_bound) = self
            .context
            .on_expression(a.lower_bound, ExprContext::Default)?;
        implicits.append(&mut implicit_variables_lower_bound);
        let (upper_bound, mut implicit_variables_upper_bound) = self
            .context
            .on_expression(a.upper_bound, ExprContext::Default)?;
        implicits.append(&mut implicit_variables_upper_bound);
        let (step, mut implicit_variables_step) = self
            .context
            .on_opt_expression(a.step, ExprContext::Default)?;
        implicits.append(&mut implicit_variables_step);
        let (next_counter, mut implicit_variables_next_counter) = self
            .context
            .on_opt_expression(a.next_counter, ExprContext::Assignment)?;
        implicits.append(&mut implicit_variables_next_counter);
        let (statements, mut implicit_variables_block) =
            self.convert_block_keeping_implicits(a.statements)?;
        implicits.append(&mut implicit_variables_block);
        Ok((
            ForLoopNode {
                variable_name,
                lower_bound,
                upper_bound,
                step,
                statements,
                next_counter,
            },
            implicits,
        ))
    }
}
