use crate::common::{AtLocation, Locatable, QError, QErrorNode, ToLocatableError};
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{ParamName, SubImplementation, TopLevelToken};
use crate::parser;
use crate::parser::{BareName, BareNameNode};

impl<'a> ConverterImpl<'a> {
    pub fn convert_sub_implementation(
        &mut self,
        sub_name_node: BareNameNode,
        params: parser::ParamNameNodes,
        block: parser::StatementNodes,
    ) -> Result<Option<TopLevelToken>, QErrorNode> {
        self.push_sub_context(&sub_name_node);
        let mut mapped_params: Vec<Locatable<ParamName>> = vec![];
        for declared_name_node in params.into_iter() {
            let Locatable {
                element: declared_name,
                pos,
            } = declared_name_node;
            let bare_name: &BareName = declared_name.as_ref();
            if self.subs.contains_key(bare_name) {
                // not possible to have a param name that clashes with a sub (functions are ok)
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
            let (param_name, _) = self
                .resolve_declared_parameter_name(&declared_name)
                .with_err_at(pos)?;
            self.context
                .push_param(declared_name, &self.resolver)
                .with_err_at(pos)?;
            mapped_params.push(param_name.at(pos));
        }

        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name: sub_name_node,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(Some(mapped))
    }
}
