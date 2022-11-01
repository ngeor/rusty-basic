use crate::{BareName, Name, TypeQualifier};
use rusty_common::QError;

/// The resolved type of an expression.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionType {
    // TODO remove this, even if it needs a new linter type
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
}

pub trait HasExpressionType {
    fn expression_type(&self) -> ExpressionType;
}
