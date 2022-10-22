use crate::common::Locatable;
use crate::parser::pc::Token;
use crate::parser::*;

pub type ParamName = VarName<ParamType>;
pub type ParamNameNode = Locatable<ParamName>;
pub type ParamNameNodes = Vec<ParamNameNode>;

// same as dim minus the "x as string * 5" and the array dimensions
#[derive(Clone, Debug)]
pub enum ParamType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareNameNode),
    Array(Box<ParamType>),
}

impl Default for ParamType {
    fn default() -> Self {
        Self::Bare
    }
}

impl VarTypeNewBuiltInCompact for ParamType {
    fn new_built_in_compact(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Compact)
    }
}

impl VarTypeNewBuiltInExtended for ParamType {
    fn new_built_in_extended(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Extended)
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

impl VarTypeNewUserDefined for ParamType {
    fn new_user_defined(name_node: BareNameNode) -> Self {
        Self::UserDefined(name_node)
    }
}

impl VarTypeToUserDefinedRecursively for ParamType {
    fn as_user_defined_recursively(&self) -> Option<&BareNameNode> {
        match self {
            Self::UserDefined(n) => Some(n),
            Self::Array(e) => e.as_user_defined_recursively(),
            _ => None,
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
            Self::Array(boxed_element_type) => {
                ExpressionType::Array(Box::new(boxed_element_type.expression_type()))
            }
            _ => ExpressionType::Unresolved,
        }
    }
}

impl VarTypeQualifier for ParamType {
    fn to_qualifier_recursively(&self) -> Option<TypeQualifier> {
        match self {
            Self::BuiltIn(q, _) => Some(*q),
            Self::Array(e) => e.to_qualifier_recursively(),
            _ => None,
        }
    }
}

impl VarTypeIsExtended for ParamType {
    fn is_extended(&self) -> bool {
        match self {
            Self::BuiltIn(_, BuiltInStyle::Extended) | Self::UserDefined(_) => true,
            Self::Array(element_type) => element_type.is_extended(),
            _ => false,
        }
    }
}
