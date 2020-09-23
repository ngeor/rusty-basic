use super::{HasTypeDefinition, TypeDefinition};
use crate::common::Locatable;
use crate::parser::{BareName, TypeQualifier};
use std::collections::HashMap;

// ========================================================
// ResolvedParamName
// ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ParamName {
    // A -> A!
    // A AS STRING
    // A$, A% etc
    BuiltIn(BareName, TypeQualifier),

    // DIM C AS Card
    UserDefined(BareName, BareName),
}

impl AsRef<BareName> for ParamName {
    fn as_ref(&self) -> &BareName {
        match self {
            Self::BuiltIn(name, _) | Self::UserDefined(name, _) => name,
        }
    }
}

impl HasTypeDefinition for ParamName {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::BuiltIn(_, qualifier) => TypeDefinition::BuiltIn(*qualifier),
            Self::UserDefined(_, type_name) => TypeDefinition::UserDefined(type_name.clone()),
        }
    }
}

// ========================================================
// ParamTypeDefinition
// ========================================================

#[derive(Clone, Debug, PartialEq)]
pub enum ParamTypeDefinition {
    BuiltIn(TypeQualifier),
    UserDefined(BareName),
}

pub type ParamTypes = Vec<ParamTypeDefinition>;

impl PartialEq<TypeDefinition> for ParamTypeDefinition {
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

// ========================================================
// SubMap, FunctionMap
// ========================================================

pub type SubSignature = ParamTypes;
pub type SubSignatureNode = Locatable<SubSignature>;
pub type SubMap = HashMap<BareName, SubSignatureNode>;

pub type FunctionSignature = (TypeQualifier, ParamTypes);
pub type FunctionSignatureNode = Locatable<FunctionSignature>;
pub type FunctionMap = HashMap<BareName, FunctionSignatureNode>;
