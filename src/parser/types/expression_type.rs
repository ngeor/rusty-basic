use crate::common::Locatable;
use crate::parser::{BareName, TypeQualifier};

/// The resolved type of an expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExpressionType {
    Unresolved,
    BuiltIn(TypeQualifier),
    FixedLengthString(u16),
    UserDefined(BareName),
    Array(Box<ExpressionType>),
}

pub trait HasExpressionType {
    fn expression_type(&self) -> ExpressionType;
}

impl<T: HasExpressionType> HasExpressionType for Locatable<T> {
    fn expression_type(&self) -> ExpressionType {
        self.as_ref().expression_type()
    }
}
