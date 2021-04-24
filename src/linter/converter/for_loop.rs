use crate::linter::converter::conversion_traits::{
    SameTypeConverter, SameTypeConverterWithImplicits,
};
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::ForLoopNode;

impl<'a> SameTypeConverterWithImplicits<ForLoopNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: ForLoopNode) -> R<ForLoopNode> {
        let (variable_name, implicit_variables_variable_name) = self
            .context
            .on_expression(a.variable_name, ExprContext::Assignment)?;
        let (lower_bound, implicit_variables_lower_bound) = self
            .context
            .on_expression(a.lower_bound, ExprContext::Default)?;
        let (upper_bound, implicit_variables_upper_bound) = self
            .context
            .on_expression(a.upper_bound, ExprContext::Default)?;
        let (step, implicit_variables_step) = self
            .context
            .on_opt_expression(a.step, ExprContext::Default)?;
        let (next_counter, implicit_variables_next_counter) = self
            .context
            .on_opt_expression(a.next_counter, ExprContext::Assignment)?;
        let implicit_vars = Self::merge_implicit_vars(vec![
            implicit_variables_variable_name,
            implicit_variables_lower_bound,
            implicit_variables_upper_bound,
            implicit_variables_step,
            implicit_variables_next_counter,
        ]);

        Ok((
            ForLoopNode {
                variable_name,
                lower_bound,
                upper_bound,
                step,
                statements: self.convert_same_type(a.statements)?,
                next_counter,
            },
            implicit_vars,
        ))
    }
}
