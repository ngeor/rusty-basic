use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::ConverterImpl;
use crate::parser::{FunctionImplementation, TopLevelToken};

impl ConverterImpl {
    pub fn convert_function_implementation(
        &mut self,
        function_implementation: FunctionImplementation,
    ) -> Result<TopLevelToken, QErrorNode> {
        let FunctionImplementation {
            name:
                Locatable {
                    element: unresolved_function_name,
                    pos,
                },
            params,
            body,
            is_static,
        } = function_implementation;
        let (resolved_function_name, resolved_params) = self
            .context
            .push_function_context(unresolved_function_name, params)?;
        let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
            name: resolved_function_name.at(pos),
            params: resolved_params,
            body: self.convert_block_hoisting_implicits(body)?,
            is_static,
        });
        self.context.pop_context();
        Ok(mapped)
    }
}
