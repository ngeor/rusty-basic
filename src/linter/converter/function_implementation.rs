use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::{FunctionImplementation, TopLevelToken};
use crate::parser;
use crate::parser::{BareName, Name, NameNode, QualifiedName};

impl<'a> ConverterImpl<'a> {
    pub fn convert_function_implementation(
        &mut self,
        function_name_node: NameNode,
        params: parser::ParamNameNodes,
        block: parser::StatementNodes,
    ) -> Result<TopLevelToken, QErrorNode> {
        let Locatable {
            element: unresolved_function_name,
            pos,
        } = function_name_node;
        let bare_function_name: &BareName = unresolved_function_name.as_ref();
        self.push_function_context(bare_function_name.clone());
        let function_name: QualifiedName = self.resolver.resolve_name(&unresolved_function_name);
        let mapped_params = self.resolve_params(params, Some(&function_name))?;
        let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
            name: Name::Qualified(function_name).at(pos),
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(mapped)
    }
}
