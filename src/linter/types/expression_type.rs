use crate::common::CanCastTo;
use crate::parser::{BareName, Operator, TypeQualifier};

/// The resolved type of an expression.
/// It's similar to `DimType`, but it includes `FileHandle` and holds only the leaf type of a member expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExpressionType {
    BuiltIn(TypeQualifier),
    FixedLengthString(u16),
    UserDefined(BareName),
    FileHandle,
}

impl ExpressionType {
    pub fn cast_binary_op(&self, right: ExpressionType, op: Operator) -> Option<ExpressionType> {
        match self {
            ExpressionType::BuiltIn(q_left) => match right {
                ExpressionType::BuiltIn(q_right) => q_left
                    .cast_binary_op(q_right, op)
                    .map(|q_result| ExpressionType::BuiltIn(q_result)),
                ExpressionType::FixedLengthString(_) => q_left
                    .cast_binary_op(TypeQualifier::DollarString, op)
                    .map(|q_result| ExpressionType::BuiltIn(q_result)),
                _ => None,
            },
            ExpressionType::FixedLengthString(_) => match right {
                ExpressionType::BuiltIn(TypeQualifier::DollarString)
                | ExpressionType::FixedLengthString(_) => TypeQualifier::DollarString
                    .cast_binary_op(TypeQualifier::DollarString, op)
                    .map(|q_result| ExpressionType::BuiltIn(q_result)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl CanCastTo<&ExpressionType> for ExpressionType {
    fn can_cast_to(&self, other: &Self) -> bool {
        match self {
            Self::BuiltIn(q_left) => match other {
                Self::BuiltIn(q_right) => q_left.can_cast_to(*q_right),
                Self::FixedLengthString(_) => *q_left == TypeQualifier::DollarString,
                _ => false,
            },
            Self::FixedLengthString(_) => match other {
                Self::BuiltIn(TypeQualifier::DollarString) | Self::FixedLengthString(_) => true,
                _ => false,
            },
            Self::UserDefined(u_left) => match other {
                Self::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
            Self::FileHandle => false,
        }
    }
}

impl CanCastTo<TypeQualifier> for ExpressionType {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        match self {
            Self::BuiltIn(q_left) => q_left.can_cast_to(other),
            Self::FixedLengthString(_) => other == TypeQualifier::DollarString,
            _ => false,
        }
    }
}
