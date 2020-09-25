use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::{FunctionImplementation, TopLevelToken};
use crate::parser;
use crate::parser::{NameNode, QualifiedName};

impl<'a> ConverterImpl<'a> {
    pub fn convert_function_implementation(
        &mut self,
        function_name_node: NameNode,
        params: parser::ParamNameNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        self.push_function_context(&function_name_node);
        let Locatable {
            element: unresolved_function_name,
            pos,
        } = function_name_node;
        let function_name: QualifiedName = self.resolver.resolve_name(&unresolved_function_name);
        let mapped_params = self.resolve_params(params, Some(&function_name))?;
        let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
            name: function_name.at(pos),
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(Some(mapped))
    }
}
