use crate::common::QError;
use crate::parser::{
    ArrayDimensions, BareNameNode, BuiltInStyle, ExpressionNode, ExpressionType, HasExpressionType,
    TypeQualifier,
};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    FixedLengthString(ExpressionNode, u16),
    UserDefined(BareNameNode),
    Array(ArrayDimensions, Box<DimType>),
}

pub trait DimTypeTrait {
    fn is_extended(&self) -> bool;

    fn is_user_defined(&self) -> Option<&BareNameNode>;
}

impl DimTypeTrait for DimType {
    fn is_extended(&self) -> bool {
        match self {
            Self::BuiltIn(_, BuiltInStyle::Extended)
            | Self::FixedLengthString(_, _)
            | Self::UserDefined(_) => true,
            Self::Array(_, element_type) => element_type.is_extended(),
            _ => false,
        }
    }

    fn is_user_defined(&self) -> Option<&BareNameNode> {
        match self {
            Self::UserDefined(u) => Some(u),
            Self::Array(_, element_type) => element_type.is_user_defined(),
            _ => None,
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

impl TryFrom<&DimType> for TypeQualifier {
    type Error = QError;

    fn try_from(value: &DimType) -> Result<Self, Self::Error> {
        let opt_q: Option<TypeQualifier> = value.into();
        opt_q.ok_or(QError::TypeMismatch)
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
