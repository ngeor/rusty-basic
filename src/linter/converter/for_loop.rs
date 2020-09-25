use crate::common::{AtLocation, Locatable, QErrorNode, ToLocatableError};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{DimName, DimNameNode, ForLoopNode};
use crate::parser;
use crate::parser::NameNode;

impl<'a> Converter<parser::ForLoopNode, ForLoopNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
        Ok(ForLoopNode {
            variable_name: self.for_loop_variable_name(a.variable_name)?,
            lower_bound: self.convert(a.lower_bound)?,
            upper_bound: self.convert(a.upper_bound)?,
            step: self.convert(a.step)?,
            statements: self.convert(a.statements)?,
            next_counter: self.for_loop_next_counter(a.next_counter)?,
        })
    }
}

impl<'a> ConverterImpl<'a> {
    fn for_loop_variable_name(&mut self, name_node: NameNode) -> Result<DimName, QErrorNode> {
        let Locatable { element, pos } = name_node;
        self.assignment_name(element).with_err_at(pos)
    }

    fn for_loop_next_counter(
        &mut self,
        opt_name_node: Option<NameNode>,
    ) -> Result<Option<DimNameNode>, QErrorNode> {
        match opt_name_node {
            Some(name_node) => {
                let Locatable { element, pos } = name_node;
                let dim_name = self.assignment_name(element).with_err_at(pos)?;
                Ok(Some(dim_name.at(pos)))
            }
            None => Ok(None),
        }
    }
}
