use crate::common::QErrorNode;
use crate::linter::converter::Context;
use crate::parser::{SubImplementation, TopLevelToken};

impl Context {
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
        let mapped_params = self.push_sub_context(params)?;
        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name,
            params: mapped_params,
            body: self.convert_block_hoisting_implicits(body)?,
            is_static,
        });
        Ok(mapped)
    }
}
