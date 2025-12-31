use rusty_common::{AtPos, Position};

use crate::{
    ArrayDimensions, BareNamePos, BuiltInStyle, Expression, ExpressionPos, ExpressionType, HasExpressionType, TypeQualifier, VarType
};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum DimType {
    #[default]
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    FixedLengthString(ExpressionPos, u16),
    UserDefined(BareNamePos),
    Array(ArrayDimensions, Box<Self>),
}

impl VarType for DimType {
    fn new_built_in_compact(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Compact)
    }

    fn new_built_in_extended(q: TypeQualifier) -> Self {
        Self::BuiltIn(q, BuiltInStyle::Extended)
    }

    fn new_user_defined(bare_name_pos: BareNamePos) -> Self {
        Self::UserDefined(bare_name_pos)
    }

    fn as_user_defined_recursively(&self) -> Option<&BareNamePos> {
        match self {
            Self::UserDefined(n) => Some(n),
            Self::Array(_, e) => e.as_user_defined_recursively(),
            _ => None,
        }
    }

    fn to_qualifier_recursively(&self) -> Option<TypeQualifier> {
        match self {
            Self::BuiltIn(q, _) => Some(*q),
            Self::FixedLengthString(_, _) => Some(TypeQualifier::DollarString),
            Self::Array(_, e) => e.to_qualifier_recursively(),
            _ => None,
        }
    }

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

impl DimType {
    pub fn fixed_length_string(len: u16, pos: Position) -> Self {
        Self::FixedLengthString(Expression::IntegerLiteral(len as i32).at_pos(pos), len)
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
