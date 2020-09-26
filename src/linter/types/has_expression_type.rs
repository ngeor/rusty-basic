use super::ExpressionType;
use crate::common::Locatable;

pub trait HasExpressionType {
    fn expression_type(&self) -> ExpressionType;
}

impl<T: HasExpressionType> HasExpressionType for Locatable<T> {
    fn expression_type(&self) -> ExpressionType {
        self.as_ref().expression_type()
    }
}
