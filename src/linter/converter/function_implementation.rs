use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::parser::{
    FunctionImplementation, NameNode, ParamNameNodes, StatementNodes, TopLevelToken,
};

impl<'a> ConverterImpl<'a> {
    pub fn convert_function_implementation(
        &mut self,
        function_name_node: NameNode,
        params: ParamNameNodes,
        block: StatementNodes,
    ) -> Result<TopLevelToken, QErrorNode> {
        let Locatable {
            element: unresolved_function_name,
            pos,
        } = function_name_node;
        let (resolved_function_name, resolved_params) = self
            .context
            .push_function_context(unresolved_function_name, params)?;
        let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
            name: resolved_function_name.at(pos),
            params: resolved_params,
            body: self.convert(block)?,
        });
        self.context.pop_context();
        Ok(mapped)
    }
}
