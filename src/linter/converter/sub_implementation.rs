use crate::common::QErrorNode;
use crate::linter::converter::{Converter, ConverterImpl};
use crate::parser::{SubImplementation, TopLevelToken};

impl<'a> ConverterImpl<'a> {
    pub fn convert_sub_implementation(
        &mut self,
        sub_implementation: SubImplementation,
    ) -> Result<TopLevelToken, QErrorNode> {
        let SubImplementation {
            name,
            params,
            body,
            is_static,
        } = sub_implementation;
        let mapped_params = self.context.push_sub_context(params)?;
        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name,
            params: mapped_params,
            body: self.convert(body)?,
            is_static,
        });
        self.context.pop_context();
        Ok(mapped)
    }
}
