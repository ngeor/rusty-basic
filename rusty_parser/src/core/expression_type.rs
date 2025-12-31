use crate::{BareName, TypeQualifier};

/// The resolved type of an expression.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionType {
    // TODO remove this, even if it needs a new linter type
    Unresolved,
    BuiltIn(TypeQualifier),
    FixedLengthString(u16),
    UserDefined(BareName),
    Array(Box<Self>),
}

impl ExpressionType {
    pub fn opt_qualifier(&self) -> Option<TypeQualifier> {
        match self {
            Self::BuiltIn(expr_q) => Some(*expr_q),
            Self::FixedLengthString(_) => Some(TypeQualifier::DollarString),
            Self::Array(boxed_expr_type) => boxed_expr_type.opt_qualifier(),
            _ => None,
        }
    }
}

pub trait HasExpressionType {
    fn expression_type(&self) -> ExpressionType;
}
