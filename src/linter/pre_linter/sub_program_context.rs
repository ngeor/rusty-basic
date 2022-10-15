use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::pre_linter::context::MainContextWithPos;
use crate::linter::type_resolver::{IntoTypeQualifier, TypeResolver};
use crate::linter::{FunctionSignature, ParamTypes, SubSignature};
use crate::parser::{BareName, Name, ParamNameNodes};
use std::collections::HashMap;

pub struct SubprogramContext<T> {
    declarations: HashMap<CaseInsensitiveString, Locatable<T>>,
    implementations: HashMap<CaseInsensitiveString, Locatable<T>>,
}

pub type FunctionContext = SubprogramContext<FunctionSignature>;
pub type SubContext = SubprogramContext<SubSignature>;

trait CheckSignature<T>
where
    T: PartialEq,
{
    /// Checks the signature of the given subprogram name against already known definitions.
    /// Returns an error if the signature doesn't match.
    /// Returns true if the definition already exists.
    /// Returns false if the definition doesn't exist.
    fn check_signature(&self, name: &BareName, signature: &T) -> Result<bool, QError>;
}

impl<T> CheckSignature<T> for HashMap<CaseInsensitiveString, Locatable<T>>
where
    T: PartialEq,
{
    fn check_signature(&self, name: &BareName, signature: &T) -> Result<bool, QError> {
        if let Some(Locatable { element, .. }) = self.get(name) {
            if element != signature {
                Err(QError::TypeMismatch)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

/// Converts a sub-program name into a sub-program signature.
pub trait ToSignature {
    type Signature;

    fn to_signature(
        &self,
        resolver: &impl TypeResolver,
        qualified_params: ParamTypes,
    ) -> Self::Signature;
}

impl ToSignature for BareName {
    type Signature = SubSignature;

    fn to_signature(
        &self,
        _resolver: &impl TypeResolver,
        qualified_params: ParamTypes,
    ) -> Self::Signature {
        qualified_params
    }
}

impl ToSignature for Name {
    type Signature = FunctionSignature;

    fn to_signature(
        &self,
        resolver: &impl TypeResolver,
        qualified_params: ParamTypes,
    ) -> Self::Signature {
        let q = self.qualify(resolver);
        (q, qualified_params)
    }
}

impl<T> SubprogramContext<T>
where
    T: PartialEq,
{
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            implementations: HashMap::new(),
        }
    }

    pub fn add_declaration<N: AsRef<BareName> + ToSignature<Signature = T>>(
        &mut self,
        name_node: &Locatable<N>,
        param_name_nodes: &ParamNameNodes,
        context: &MainContextWithPos,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: context,
            pos: declaration_pos,
        } = context;
        let Locatable { element: name, pos } = name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let param_types: ParamTypes = params::resolve_param_types(context, param_name_nodes)?;
        let bare_name: &BareName = name.as_ref();
        let signature = name.to_signature(context, param_types);
        self.implementations
            .check_signature(bare_name, &signature)
            .with_err_at(pos)?;
        if !self
            .declarations
            .check_signature(bare_name, &signature)
            .with_err_at(pos)?
        {
            self.declarations
                .insert(bare_name.clone(), signature.at(*declaration_pos));
        }
        Ok(())
    }

    pub fn add_implementation<N: AsRef<BareName> + ToSignature<Signature = T>>(
        &mut self,
        name_node: &Locatable<N>,
        param_name_nodes: &ParamNameNodes,
        context: &MainContextWithPos,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: context,
            pos: implementation_pos,
        } = context;
        let Locatable { element: name, pos } = name_node;

        // type must match declaration
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let param_types: ParamTypes = params::resolve_param_types(context, param_name_nodes)?;
        let bare_name: &BareName = name.as_ref();
        let signature = name.to_signature(context, param_types);
        match self.implementations.get(bare_name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
            None => {
                self.declarations
                    .check_signature(bare_name, &signature)
                    .with_err_at(pos)?;
                self.implementations
                    .insert(bare_name.clone(), signature.at(*implementation_pos));
                Ok(())
            }
        }
    }

    pub fn implementations(self) -> HashMap<CaseInsensitiveString, Locatable<T>> {
        self.implementations
    }

    fn ensure_declarations_are_implemented(&self) -> Result<(), QErrorNode> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v);
            }
        }
        Ok(())
    }

    fn ensure_does_not_clash_with_built_in<F>(&self, is_built_in: F) -> Result<(), QErrorNode>
    where
        F: Fn(&BareName) -> bool,
    {
        for (k, v) in self.implementations.iter() {
            if is_built_in(k) {
                return Err(QError::DuplicateDefinition).with_err_at(v);
            }
        }

        Ok(())
    }
}

pub trait PostVisit {
    fn post_visit(&self) -> Result<(), QErrorNode>;
}

impl PostVisit for FunctionContext {
    fn post_visit(&self) -> Result<(), QErrorNode> {
        self.ensure_declarations_are_implemented()?;
        self.ensure_does_not_clash_with_built_in(|name| {
            Option::<BuiltInFunction>::from(name).is_some()
        })
    }
}

impl PostVisit for SubContext {
    fn post_visit(&self) -> Result<(), QErrorNode> {
        // not checking if declarations are present, because in MONEY.BAS there
        // are two SUBs declared but not implemented (and not called either)
        self.ensure_does_not_clash_with_built_in(|name| {
            BuiltInSub::parse_non_keyword_sub(name.as_ref()).is_some()
        })
    }
}

mod params {
    //! resolving function/sub parameters

    use crate::common::{AtLocation, Locatable, QError, QErrorNode, ToLocatableError};
    use crate::linter::pre_linter::context::MainContext;
    use crate::linter::type_resolver::IntoTypeQualifier;
    use crate::parser::{
        BareName, BuiltInStyle, ParamName, ParamNameNode, ParamNameNodes, ParamType,
    };

    pub fn resolve_param_types(
        context: &MainContext,
        params: &ParamNameNodes,
    ) -> Result<Vec<ParamType>, QErrorNode> {
        params.iter().map(|p| parameter(context, p)).collect()
    }

    /// Resolves bare parameter types to qualified and ensures user defined types exist.
    fn parameter(context: &MainContext, param: &ParamNameNode) -> Result<ParamType, QErrorNode> {
        let Locatable {
            element: param,
            pos,
        } = param;
        let bare_name: &BareName = param.bare_name();
        match &param.var_type {
            ParamType::Bare => {
                let q = bare_name.qualify(context);
                Ok(ParamType::BuiltIn(q, BuiltInStyle::Compact))
            }
            ParamType::BuiltIn(q, built_in_style) => Ok(ParamType::BuiltIn(*q, *built_in_style)),
            ParamType::UserDefined(u) => {
                let type_name: &BareName = u.as_ref();
                if context.user_defined_types().contains_key(type_name) {
                    Ok(ParamType::UserDefined(u.clone()))
                } else {
                    Err(QError::TypeNotDefined).with_err_at(pos)
                }
            }
            ParamType::Array(element_type) => {
                let dummy_element_param =
                    ParamName::new(bare_name.clone(), element_type.as_ref().clone()).at(pos);
                let element_param_type = parameter(context, &dummy_element_param)?;
                Ok(ParamType::Array(Box::new(element_param_type)))
            }
        }
    }
}
