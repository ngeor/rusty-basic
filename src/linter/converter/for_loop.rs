use crate::common::{AtLocation, HasLocation, Locatable, QErrorNode, ToLocatableError};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{DimName, ForLoopNode};
use crate::parser;
use crate::parser::NameNode;

impl<'a> Converter<parser::ForLoopNode, ForLoopNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::ForLoopNode) -> Result<ForLoopNode, QErrorNode> {
        Ok(ForLoopNode {
            variable_name: self.temp_convert(a.variable_name)?,
            lower_bound: self.convert(a.lower_bound)?,
            upper_bound: self.convert(a.upper_bound)?,
            step: self.convert(a.step)?,
            statements: self.convert(a.statements)?,
            next_counter: match a.next_counter {
                Some(x) => {
                    let pos = x.pos();
                    Some(self.temp_convert(x)?.at(pos))
                }
                None => None,
            },
        })
    }
}

impl<'a> ConverterImpl<'a> {
    // TODO fix me
    fn temp_convert(&mut self, x: NameNode) -> Result<DimName, QErrorNode> {
        let Locatable { element, pos } = x;
        self.resolve_name_in_assignment(element).with_err_at(pos)
    }
}
