use crate::common::*;
use crate::parser::pc::Token;
use crate::parser::types::*;
use std::collections::HashMap;

pub type ParamName = VarName<ParamType>;
pub type ParamNameNode = Locatable<ParamName>;
pub type ParamNameNodes = Vec<ParamNameNode>;

// same as dim minus the x as string * 5 and the array dimensions
#[derive(Clone, Debug)]
pub enum ParamType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareNameNode),
    Array(Box<ParamType>),
}

pub type ParamTypes = Vec<ParamType>;

impl Default for ParamType {
    fn default() -> Self {
        Self::Bare
    }
}

impl From<TypeQualifier> for ParamType {
    fn from(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Compact)
    }
}

impl VarTypeToArray for ParamType {
    type ArrayType = Option<(Token, Token)>;

    fn to_array(self, array_type: Self::ArrayType) -> Self {
        if array_type.is_none() {
            self
        } else {
            Self::Array(Box::new(self))
        }
    }
}

impl VarTypeUserDefined for ParamType {
    fn from_user_defined(name_node: BareNameNode) -> Self {
        Self::UserDefined(name_node)
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
            Self::Array(boxed_element_type) => {
                ExpressionType::Array(Box::new(boxed_element_type.expression_type()))
            }
            _ => ExpressionType::Unresolved,
        }
    }
}

pub type SubSignature = ParamTypes;
pub type SubSignatureNode = Locatable<SubSignature>;
pub type SubMap = HashMap<BareName, SubSignatureNode>;

pub type FunctionSignature = (TypeQualifier, ParamTypes);
pub type FunctionSignatureNode = Locatable<FunctionSignature>;
pub type FunctionMap = HashMap<BareName, FunctionSignatureNode>;

impl From<ParamType> for DimType {
    fn from(param_type: ParamType) -> Self {
        match param_type {
            ParamType::Bare => DimType::Bare,
            ParamType::BuiltIn(q, built_in_style) => DimType::BuiltIn(q, built_in_style),
            ParamType::UserDefined(user_defined_type_name_node) => {
                DimType::UserDefined(user_defined_type_name_node)
            }
            ParamType::Array(boxed_element_type) => {
                DimType::Array(vec![], Box::new(Self::from(*boxed_element_type)))
            }
        }
    }
}

impl From<DimType> for ParamType {
    fn from(dim_type: DimType) -> Self {
        match dim_type {
            DimType::Bare => ParamType::Bare,
            DimType::BuiltIn(q, built_in_style) => ParamType::BuiltIn(q, built_in_style),
            DimType::UserDefined(user_defined_type_name_node) => {
                ParamType::UserDefined(user_defined_type_name_node)
            }
            DimType::Array(_, boxed_element_type) => {
                ParamType::Array(Box::new(Self::from(*boxed_element_type)))
            }
            DimType::FixedLengthString(_, _) => {
                panic!("Fixed length string params are not supported")
            }
        }
    }
}
