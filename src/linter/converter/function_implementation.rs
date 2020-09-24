use crate::common::{AtLocation, Locatable, QError, QErrorNode, ToLocatableError};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::type_resolver::ResolveInto;
use crate::linter::{FunctionImplementation, ParamName, ParamType, TopLevelToken};
use crate::parser;
use crate::parser::{BareName, HasQualifier, NameNode, QualifiedName, QualifiedNameNode};

impl<'a> ConverterImpl<'a> {
    pub fn convert_function_implementation(
        &mut self,
        function_name_node: NameNode,
        params: parser::ParamNameNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        let mapped_name: QualifiedNameNode =
            function_name_node.map(|x| x.resolve_into(&self.resolver));
        self.push_function_context(mapped_name.as_ref());
        let mapped_params = self.convert_function_params(mapped_name.as_ref(), params)?;
        let mapped = TopLevelToken::FunctionImplementation(FunctionImplementation {
            name: mapped_name,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(Some(mapped))
    }

    fn convert_function_params(
        &mut self,
        function_name: &QualifiedName,
        params: parser::ParamNameNodes,
    ) -> Result<Vec<Locatable<ParamName>>, QErrorNode> {
        let mut result: Vec<Locatable<ParamName>> = vec![];
        for p in params.into_iter() {
            let Locatable {
                element: declared_name,
                pos,
            } = p;
            let bare_name: &BareName = declared_name.as_ref();
            if self.subs.contains_key(bare_name) {
                // not possible to have a param name that clashes with a sub (functions are ok)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let (param_name, is_extended) = self
                .resolve_declared_parameter_name(&declared_name)
                .with_err_at(pos)?;
            let bare_function_name: &BareName = function_name.as_ref();
            if bare_function_name == bare_name {
                // not possible to have a param name clashing with the function name if the type is different or if it's an extended declaration (AS SINGLE)
                let clashes = match param_name.param_type() {
                    ParamType::BuiltIn(qualifier) => {
                        *qualifier != function_name.qualifier() || is_extended
                    }
                    _ => true,
                };
                if clashes {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
            }
            self.context
                .push_param(declared_name, &self.resolver)
                .with_err_at(pos)?;
            result.push(param_name.at(pos));
        }
        Ok(result)
    }
}
