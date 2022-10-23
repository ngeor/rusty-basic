use crate::common::Locatable;
use crate::parser::pc::Token;
use crate::parser::*;

pub type ParamName = VarName<ParamType>;
pub type ParamNameNode = Locatable<ParamName>;
pub type ParamNameNodes = Vec<ParamNameNode>;

// same as dim minus the "x as string * 5" and the array dimensions
#[derive(Clone, Debug, Eq, PartialEq)]
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
