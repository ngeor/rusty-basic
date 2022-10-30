use crate::{
    ArrayDimensions, BareNameNode, BuiltInStyle, Expression, ExpressionNode, ExpressionType,
    HasExpressionType, TypeQualifier, VarTypeIsExtended, VarTypeNewBuiltInCompact,
    VarTypeNewBuiltInExtended, VarTypeNewUserDefined, VarTypeQualifier, VarTypeToArray,
    VarTypeToUserDefinedRecursively,
};
use rusty_common::{AtLocation, Location};

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    FixedLengthString(ExpressionNode, u16),
    UserDefined(BareNameNode),
    Array(ArrayDimensions, Box<DimType>),
}

impl Default for DimType {
    fn default() -> Self {
        Self::Bare
    }
}

impl VarTypeNewBuiltInCompact for DimType {
    fn new_built_in_compact(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Compact)
    }
}

impl VarTypeNewBuiltInExtended for DimType {
    fn new_built_in_extended(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Extended)
    }
}

impl VarTypeToArray for DimType {
    type ArrayType = ArrayDimensions;

    fn to_array(self, array_type: Self::ArrayType) -> Self {
        if array_type.is_empty() {
            self
        } else {
            Self::Array(array_type, Box::new(self))
        }
    }
}

impl VarTypeNewUserDefined for DimType {
    fn new_user_defined(name_node: BareNameNode) -> Self {
        Self::UserDefined(name_node)
    }
}

impl VarTypeToUserDefinedRecursively for DimType {
    fn as_user_defined_recursively(&self) -> Option<&BareNameNode> {
        match self {
            Self::UserDefined(n) => Some(n),
            Self::Array(_, e) => e.as_user_defined_recursively(),
            _ => None,
        }
    }
}

impl DimType {
    pub fn fixed_length_string(len: u16, pos: Location) -> Self {
        DimType::FixedLengthString(Expression::IntegerLiteral(len as i32).at(pos), len)
    }
}

impl VarTypeIsExtended for DimType {
    fn is_extended(&self) -> bool {
        match self {
            Self::BuiltIn(_, BuiltInStyle::Extended)
            | Self::FixedLengthString(_, _)
            | Self::UserDefined(_) => true,
            Self::Array(_, element_type) => element_type.is_extended(),
            _ => false,
        }
    }
}

impl HasExpressionType for DimType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier, _) => ExpressionType::BuiltIn(*qualifier),
            Self::FixedLengthString(_, len) => ExpressionType::FixedLengthString(*len),
            Self::UserDefined(type_name) => ExpressionType::UserDefined(type_name.element.clone()),
            Self::Array(_, element_type) => {
                ExpressionType::Array(Box::new(element_type.expression_type()))
            }
            Self::Bare => panic!("Unresolved type"),
        }
    }
}

impl VarTypeQualifier for DimType {
    fn to_qualifier_recursively(&self) -> Option<TypeQualifier> {
        match self {
            Self::BuiltIn(q, _) => Some(*q),
            Self::FixedLengthString(_, _) => Some(TypeQualifier::DollarString),
            Self::Array(_, e) => e.to_qualifier_recursively(),
            _ => None,
        }
    }
}
