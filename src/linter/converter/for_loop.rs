use crate::common::QErrorNode;
use crate::linter::converter::converter::{
    Converter, ConverterImpl, ConverterWithImplicitVariables,
};
use crate::parser::{ForLoopNode, QualifiedNameNode};

impl<'a> ConverterWithImplicitVariables<ForLoopNode, ForLoopNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: ForLoopNode,
    ) -> Result<(ForLoopNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (variable_name, implicit_variables_variable_name) =
            self.convert_and_collect_implicit_variables(a.variable_name)?;
        let (lower_bound, implicit_variables_lower_bound) =
            self.convert_and_collect_implicit_variables(a.lower_bound)?;
        let (upper_bound, implicit_variables_upper_bound) =
            self.convert_and_collect_implicit_variables(a.upper_bound)?;
        let (step, implicit_variables_step) =
            self.convert_and_collect_implicit_variables(a.step)?;
        let (next_counter, implicit_variables_next_counter) =
            self.convert_and_collect_implicit_variables(a.next_counter)?;

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
                statements: self.convert(a.statements)?,
                next_counter,
            },
            implicit_vars,
        ))
    }
}
