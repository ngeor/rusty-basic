use super::{ExpressionType, HasExpressionType};
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

impl HasExpressionType for ParamName {
    fn expression_type(&self) -> ExpressionType {
        self.param_type.expression_type()
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

impl PartialEq<ExpressionType> for ParamType {
    fn eq(&self, type_definition: &ExpressionType) -> bool {
        match self {
            Self::BuiltIn(q_left) => match type_definition {
                ExpressionType::BuiltIn(q_right) => q_left == q_right,
                _ => false,
            },
            Self::UserDefined(u_left) => match type_definition {
                ExpressionType::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
        }
    }
}

impl HasExpressionType for ParamType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier) => ExpressionType::BuiltIn(*qualifier),
            Self::UserDefined(type_name) => ExpressionType::UserDefined(type_name.clone()),
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
