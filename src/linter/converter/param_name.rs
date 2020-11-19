use crate::common::{
    AtLocation, Locatable, PatchErrPos, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::converter::converter::ConverterImpl;
use crate::linter::type_resolver::TypeResolver;
use crate::parser::{
    BareName, BareNameNode, BuiltInStyle, ExpressionType, Name, ParamName, ParamNameNode,
    ParamType, QualifiedName, TypeQualifier,
};

impl<'a> ConverterImpl<'a> {
    pub fn resolve_params(
        &mut self,
        params: Vec<ParamNameNode>,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<Vec<Locatable<ParamName>>, QErrorNode> {
        params
            .into_iter()
            .map(|p| self.resolve_param_node(p, opt_function_name))
            .collect()
    }

    fn resolve_param_node(
        &mut self,
        param_node: ParamNameNode,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<Locatable<ParamName>, QErrorNode> {
        let Locatable {
            element: param,
            pos,
        } = param_node;
        self.resolve_param(param, opt_function_name)
            .map(|x| x.at(pos))
            .patch_err_pos(pos)
    }

    fn resolve_param(
        &mut self,
        param: ParamName,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<ParamName, QErrorNode> {
        let (bare_name, param_type) = param.into_inner();
        // ensure does not clash with sub name
        if self.subs.contains_key(&bare_name) {
            // not possible to have a param name that clashes with a sub (functions are ok)
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }
        match param_type {
            ParamType::Bare => self.resolve_param_bare(bare_name, opt_function_name),
            ParamType::BuiltIn(q, BuiltInStyle::Compact) => {
                self.resolve_param_compact(bare_name, q, opt_function_name)
            }
            ParamType::BuiltIn(q, BuiltInStyle::Extended) => {
                self.resolve_param_extended(bare_name, q, opt_function_name)
            }
            ParamType::UserDefined(u) => {
                self.resolve_param_user_defined(bare_name, u, opt_function_name)
            }
            ParamType::Array(boxed_element_type) => match boxed_element_type.as_ref() {
                ParamType::BuiltIn(q, built_in_style) => {
                    let name = Name::new(bare_name.clone(), Some(*q));
                    let element_type = ExpressionType::BuiltIn(*q);
                    self.context
                        .register_array_dimensions(name, element_type, true);
                    Ok(ParamName::new(
                        bare_name,
                        ParamType::Array(Box::new(ParamType::BuiltIn(*q, *built_in_style))),
                    ))
                }
                _ => todo!(),
            },
        }
    }

    fn resolve_param_bare(
        &mut self,
        bare_name: BareName,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<ParamName, QErrorNode> {
        let q: TypeQualifier = self.resolve(&bare_name);
        self.resolve_param_compact(bare_name, q, opt_function_name)
    }

    fn resolve_param_compact(
        &mut self,
        bare_name: BareName,
        q: TypeQualifier,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<ParamName, QErrorNode> {
        // as we're building a new context, we know we don't have constants to check for
        if self.context.contains_extended(&bare_name)
            || self.context.contains_compact(&bare_name, q)
        {
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }
        // not possible to have a param name clashing with the function name if the type is different...
        match opt_function_name {
            Some(QualifiedName {
                bare_name: bare_function_name,
                qualifier: function_q,
            }) => {
                if &bare_name == bare_function_name && q != *function_q {
                    return Err(QError::DuplicateDefinition).with_err_no_pos();
                }
            }
            None => {}
        }
        self.context
            .push_compact_param(QualifiedName::new(bare_name.clone(), q));
        self.context.push_dim_compact(bare_name.clone(), q);
        Ok(ParamName::new(
            bare_name,
            ParamType::BuiltIn(q, BuiltInStyle::Compact),
        ))
    }

    fn resolve_param_extended(
        &mut self,
        bare_name: BareName,
        q: TypeQualifier,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<ParamName, QErrorNode> {
        if self.context.contains_any(&bare_name) {
            return Err(QError::DuplicateDefinition).with_err_no_pos();
        }
        // not possible to have a param name clashing with the function name for extended types
        match opt_function_name {
            Some(QualifiedName {
                bare_name: bare_function_name,
                ..
            }) => {
                if &bare_name == bare_function_name {
                    return Err(QError::DuplicateDefinition).with_err_no_pos();
                }
            }
            None => {}
        }
        self.context.push_extended_param(bare_name.clone());
        self.context.push_dim_extended(bare_name.clone(), q);
        Ok(ParamName::new(
            bare_name,
            ParamType::BuiltIn(q, BuiltInStyle::Extended),
        ))
    }

    fn resolve_param_user_defined(
        &mut self,
        bare_name: BareName,
        user_defined_type_name_node: BareNameNode,
        opt_function_name: Option<&QualifiedName>,
    ) -> Result<ParamName, QErrorNode> {
        if !self
            .user_defined_types
            .contains_key(user_defined_type_name_node.as_ref())
        {
            return Err(QError::TypeNotDefined).with_err_at(&user_defined_type_name_node);
        }
        // not possible to have a param name clashing with the function name for extended types
        match opt_function_name {
            Some(QualifiedName {
                bare_name: bare_function_name,
                ..
            }) => {
                if &bare_name == bare_function_name {
                    return Err(QError::DuplicateDefinition).with_err_no_pos();
                }
            }
            None => {}
        }
        let Locatable {
            element: type_name,
            pos,
        } = user_defined_type_name_node;
        self.context.push_extended_param(bare_name.clone());
        self.context
            .push_dim_user_defined(bare_name.clone(), type_name.clone());
        Ok(ParamName::new(
            bare_name,
            ParamType::UserDefined(type_name.at(pos)),
        ))
    }
}
