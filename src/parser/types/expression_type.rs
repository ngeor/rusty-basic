use crate::common::{CanCastTo, QError};
use crate::parser::{BareName, Name, Operator, QualifiedName, TypeQualifier};

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
        match name {
            Name::Bare(bare_name) => match self.opt_qualifier() {
                Some(q) => Ok(Name::new(bare_name, Some(q))),
                _ => Ok(Name::Bare(bare_name)),
            },
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => {
                match self.opt_qualifier() {
                    Some(expr_q) => {
                        if qualifier == expr_q {
                            Ok(Name::new(bare_name, Some(qualifier)))
                        } else {
                            // trying to use the wrong qualifier
                            Err(QError::TypeMismatch)
                        }
                    }
                    // trying to use a qualifier on an ExpressionType that doesn't accept it
                    _ => Err(QError::TypeMismatch),
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
