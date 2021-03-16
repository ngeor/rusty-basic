use crate::common::{CanCastTo, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use crate::parser::{
    Expression, ExpressionNodes, ExpressionType, HasExpressionType, TypeQualifier, VariableInfo,
};

pub trait ArgValidation {
    fn require_integer_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_long_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_numeric_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_string_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_string_variable(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_one_argument(&self) -> Result<(), QErrorNode>;

    fn require_one_numeric_argument(&self) -> Result<(), QErrorNode> {
        self.require_one_argument()
            .and_then(|_| self.require_numeric_argument(0))
    }

    fn require_one_string_argument(&self) -> Result<(), QErrorNode> {
        self.require_one_argument()
            .and_then(|_| self.require_string_argument(0))
    }
}

impl ArgValidation for ExpressionNodes {
    fn require_integer_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        if !self[idx].can_cast_to(TypeQualifier::PercentInteger) {
            Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx])
        } else {
            Ok(())
        }
    }

    fn require_long_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        if !self[idx].can_cast_to(TypeQualifier::AmpersandLong) {
            Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx])
        } else {
            Ok(())
        }
    }

    fn require_numeric_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        match self[idx].expression_type() {
            ExpressionType::BuiltIn(q) => {
                if q == TypeQualifier::DollarString {
                    Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx])
                } else {
                    Ok(())
                }
            }
            _ => Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx]),
        }
    }

    fn require_string_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        if !self[idx].can_cast_to(TypeQualifier::DollarString) {
            Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx])
        } else {
            Ok(())
        }
    }

    fn require_string_variable(&self, idx: usize) -> Result<(), QErrorNode> {
        match self[idx].as_ref() {
            Expression::Variable(
                _,
                VariableInfo {
                    expression_type: ExpressionType::BuiltIn(TypeQualifier::DollarString),
                    ..
                },
            ) => Ok(()),
            Expression::Variable(_, _) => Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx]),
            _ => Err(QError::VariableRequired).with_err_at(&self[idx]),
        }
    }

    fn require_one_argument(&self) -> Result<(), QErrorNode> {
        if self.len() != 1 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            Ok(())
        }
    }
}
