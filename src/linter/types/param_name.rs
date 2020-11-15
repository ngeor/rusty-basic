use super::{ExpressionType, HasExpressionType};
use crate::common::Locatable;
use crate::parser::{BareName, BareNameNode, BuiltInStyle, TypeQualifier};
use std::collections::HashMap;

// ========================================================
// ParamName
// ========================================================

#[derive(Clone, Debug, PartialEq)]
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

    pub fn into_inner(self) -> (BareName, ParamType) {
        (self.bare_name, self.param_type)
    }

    pub fn new_array(self) -> Self {
        Self::new(self.bare_name, ParamType::Array(Box::new(self.param_type)))
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

#[derive(Clone, Debug)]
pub enum ParamType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareNameNode),
    Array(Box<ParamType>),
}

pub type ParamTypes = Vec<ParamType>;

impl ParamType {
    pub fn accepts_by_ref(&self, type_definition: &ExpressionType) -> bool {
        match self {
            Self::Bare => false,
            Self::BuiltIn(q_left, _) => match type_definition {
                ExpressionType::BuiltIn(q_right) => q_left == q_right,
                ExpressionType::FixedLengthString(_) => *q_left == TypeQualifier::DollarString,
                _ => false,
            },
            Self::UserDefined(u_left) => match type_definition {
                ExpressionType::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
            Self::Array(boxed_element_type) => match type_definition {
                ExpressionType::Array(boxed_type) => boxed_element_type.accepts_by_ref(boxed_type),
                _ => false,
            },
        }
    }
}

// Custom implementation of PartialEq because we want to compare the parameter types are equal,
// regardless of the location of the UserDefinedName node. This is used in subprogram_context (pre-linter).
impl PartialEq<ParamType> for ParamType {
    fn eq(&self, other: &ParamType) -> bool {
        match self {
            Self::Bare => {
                if let Self::Bare = other {
                    true
                } else {
                    false
                }
            }
            Self::BuiltIn(q, _) => {
                if let Self::BuiltIn(q_other, _) = other {
                    q == q_other
                } else {
                    false
                }
            }
            Self::UserDefined(Locatable { element, .. }) => {
                if let Self::UserDefined(Locatable {
                    element: other_name,
                    ..
                }) = other
                {
                    element == other_name
                } else {
                    false
                }
            }
            Self::Array(boxed) => {
                if let Self::Array(boxed_other) = other {
                    boxed.as_ref() == boxed_other.as_ref()
                } else {
                    false
                }
            }
        }
    }
}

impl HasExpressionType for ParamType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier, _) => ExpressionType::BuiltIn(*qualifier),
            Self::UserDefined(Locatable { element, .. }) => {
                ExpressionType::UserDefined(element.clone())
            }
            _ => ExpressionType::Unresolved,
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
