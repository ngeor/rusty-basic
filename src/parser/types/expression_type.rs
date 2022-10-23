use crate::common::{CanCastTo, QError};
use crate::parser::{BareName, Name, Operator, TypeQualifier};

/// The resolved type of an expression.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionType {
    Unresolved,
    BuiltIn(TypeQualifier),
    FixedLengthString(u16),
    UserDefined(BareName),
    Array(Box<ExpressionType>),
}

impl ExpressionType {
    /// Validates and normalizes the given name
    pub fn qualify_name(&self, name: Name) -> Result<Name, QError> {
        match self.opt_qualifier() {
            Some(expr_q) => {
                // try to modify the name to have the expected qualifier
                name.try_qualify(expr_q).map_err(|_| QError::TypeMismatch)
            }
            None => {
                match name {
                    // trying to use a qualifier on an ExpressionType that doesn't accept it
                    Name::Qualified(_, _) => Err(QError::TypeMismatch),
                    _ => Ok(name),
                }
            }
        }
    }

    pub fn opt_qualifier(&self) -> Option<TypeQualifier> {
        match self {
            ExpressionType::BuiltIn(expr_q) => Some(*expr_q),
            ExpressionType::FixedLengthString(_) => Some(TypeQualifier::DollarString),
            ExpressionType::Array(boxed_expr_type) => boxed_expr_type.opt_qualifier(),
            _ => None,
        }
    }

    pub fn to_element_type(&self) -> &Self {
        match self {
            Self::Array(boxed_element_type) => boxed_element_type.to_element_type(),
            _ => self,
        }
    }
}

pub trait HasExpressionType {
    fn expression_type(&self) -> ExpressionType;
}

impl ExpressionType {
    pub fn cast_binary_op(&self, right: ExpressionType, op: Operator) -> Option<ExpressionType> {
        match self {
            ExpressionType::BuiltIn(q_left) => match right {
                ExpressionType::BuiltIn(q_right) => q_left
                    .cast_binary_op(q_right, op)
                    .map(ExpressionType::BuiltIn),
                ExpressionType::FixedLengthString(_) => q_left
                    .cast_binary_op(TypeQualifier::DollarString, op)
                    .map(ExpressionType::BuiltIn),
                _ => None,
            },
            ExpressionType::FixedLengthString(_) => match right {
                ExpressionType::BuiltIn(TypeQualifier::DollarString)
                | ExpressionType::FixedLengthString(_) => TypeQualifier::DollarString
                    .cast_binary_op(TypeQualifier::DollarString, op)
                    .map(ExpressionType::BuiltIn),
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
            Self::FixedLengthString(_) => matches!(
                other,
                Self::BuiltIn(TypeQualifier::DollarString) | Self::FixedLengthString(_)
            ),
            Self::UserDefined(u_left) => match other {
                Self::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
            Self::Unresolved | Self::Array(_) => false,
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
