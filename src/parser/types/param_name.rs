use crate::common::*;
use crate::parser::types::*;
use std::collections::HashMap;

// same as dim minus the x as string * 5
#[derive(Clone, Debug, PartialEq)]
pub struct ParamName {
    pub bare_name: BareName,
    pub param_type: ParamType,
}

#[derive(Clone, Debug)]
pub enum ParamType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareNameNode),
    Array(Box<ParamType>),
}

pub type ParamNameNode = Locatable<ParamName>;
pub type ParamNameNodes = Vec<ParamNameNode>;

impl ParamName {
    pub fn new(bare_name: BareName, param_type: ParamType) -> Self {
        Self {
            bare_name,
            param_type,
        }
    }

    pub fn new_array(self) -> Self {
        Self::new(self.bare_name, ParamType::Array(Box::new(self.param_type)))
    }

    pub fn to_name(&self) -> Name {
        Self::make_name(self.bare_name.clone(), self.param_type.clone())
    }

    fn make_name(bare_name: BareName, param_type: ParamType) -> Name {
        match param_type {
            ParamType::Bare | ParamType::UserDefined(_) => Name::new(bare_name, None),
            ParamType::BuiltIn(q, _) => Name::new(bare_name, Some(q)),
            ParamType::Array(boxed_element_type) => Self::make_name(bare_name, *boxed_element_type),
        }
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

pub type ParamTypes = Vec<ParamType>;

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
