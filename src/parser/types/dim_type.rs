use crate::common::{AtLocation, Location};
use crate::parser::{
    ArrayDimensions, BareNameNode, BuiltInStyle, Expression, ExpressionNode, ExpressionType,
    HasExpressionType, TypeQualifier, VarTypeToArray, VarTypeUserDefined,
};

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

impl From<TypeQualifier> for DimType {
    fn from(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Compact)
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

impl VarTypeUserDefined for DimType {
    fn from_user_defined(name_node: BareNameNode) -> Self {
        Self::UserDefined(name_node)
    }
}

impl DimType {
    pub fn fixed_length_string(len: u16, pos: Location) -> Self {
        DimType::FixedLengthString(Expression::IntegerLiteral(len as i32).at(pos), len)
    }

    pub fn is_extended(&self) -> bool {
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

impl From<&DimType> for Option<TypeQualifier> {
    fn from(dim_type: &DimType) -> Self {
        match dim_type {
            DimType::BuiltIn(q, _) => Some(*q),
            DimType::FixedLengthString(_, _) => Some(TypeQualifier::DollarString),
            DimType::Array(_, boxed_element_type) => Self::from(boxed_element_type.as_ref()),
            _ => None,
        }
    }
}
