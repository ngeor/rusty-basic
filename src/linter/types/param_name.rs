use super::{HasTypeDefinition, TypeDefinition};
use crate::common::Locatable;
use crate::parser::{BareName, TypeQualifier};
use std::collections::HashMap;

// ========================================================
// ParamName
// ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParamName {
    bare_name: BareName,
    param_type: ParamType,
}

impl ParamName {
    pub fn new(bare_name: BareName, param_type: ParamType) -> Self {
        Self {
            bare_name,
            param_type,
        }
    }

    pub fn param_type(&self) -> &ParamType {
        &self.param_type
    }
}

impl AsRef<BareName> for ParamName {
    fn as_ref(&self) -> &BareName {
        &self.bare_name
    }
}

impl HasTypeDefinition for ParamName {
    fn type_definition(&self) -> TypeDefinition {
        self.param_type.type_definition()
    }
}

// ========================================================
// ParamType
// ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ParamType {
    BuiltIn(TypeQualifier),
    UserDefined(BareName),
}

pub type ParamTypes = Vec<ParamType>;

impl PartialEq<TypeDefinition> for ParamType {
    fn eq(&self, type_definition: &TypeDefinition) -> bool {
        match self {
            Self::BuiltIn(q_left) => match type_definition {
                TypeDefinition::BuiltIn(q_right) => q_left == q_right,
                _ => false,
            },
            Self::UserDefined(u_left) => match type_definition {
                TypeDefinition::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
        }
    }
}

impl HasTypeDefinition for ParamType {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::BuiltIn(qualifier) => TypeDefinition::BuiltIn(*qualifier),
            Self::UserDefined(type_name) => TypeDefinition::UserDefined(type_name.clone()),
        }
    }
}

// ========================================================
// SubMap, FunctionMap
// ========================================================

pub type SubSignature = ParamTypes;
pub type SubSignatureNode = Locatable<SubSignature>;
pub type SubMap = HashMap<BareName, SubSignatureNode>;

pub type FunctionSignature = (TypeQualifier, ParamTypes);
pub type FunctionSignatureNode = Locatable<FunctionSignature>;
pub type FunctionMap = HashMap<BareName, FunctionSignatureNode>;
