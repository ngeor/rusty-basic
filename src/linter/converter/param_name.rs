use crate::common::QError;
use crate::linter::converter::converter::ConverterImpl;
use crate::linter::type_resolver::ResolveInto;
use crate::linter::{ParamName, ParamType};
use crate::parser;
use crate::parser::{BareName, TypeQualifier};

impl<'a> ConverterImpl<'a> {
    // TODO trait FromWithContext fn from(other, &ctx) -> Self
    // TODO the bool represents extended type, improve this
    // TODO use linter context there are a lot of similarities
    pub fn resolve_declared_parameter_name(
        &mut self,
        param: &parser::ParamName,
    ) -> Result<(ParamName, bool), QError> {
        let bare_name: &BareName = param.as_ref();
        match param.param_type() {
            parser::ParamType::Bare => {
                let q: TypeQualifier = bare_name.resolve_into(&self.resolver);
                Ok((
                    ParamName::new(bare_name.clone(), ParamType::BuiltIn(q)),
                    false,
                ))
            }
            parser::ParamType::Compact(q) => Ok((
                ParamName::new(bare_name.clone(), ParamType::BuiltIn(*q)),
                false,
            )),
            parser::ParamType::Extended(q) => Ok((
                ParamName::new(bare_name.clone(), ParamType::BuiltIn(*q)),
                true,
            )),
            parser::ParamType::UserDefined(u) => {
                let type_name: &BareName = u.as_ref();
                if self.user_defined_types.contains_key(type_name) {
                    Ok((
                        ParamName::new(
                            bare_name.clone(),
                            ParamType::UserDefined(type_name.clone()),
                        ),
                        true,
                    ))
                } else {
                    Err(QError::TypeNotDefined)
                }
            }
        }
    }
}
