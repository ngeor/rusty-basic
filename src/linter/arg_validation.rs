use crate::common::{CanCastTo, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use crate::parser::{
    Expression, ExpressionNode, ExpressionNodes, ExpressionType, HasExpressionType, TypeQualifier,
    VariableInfo,
};

pub trait ArgValidation {
    fn require_integer_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_long_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_double_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_numeric_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_string_argument(&self, idx: usize) -> Result<(), QErrorNode>;

    /// Demands that the argument at the given index is a string variable.
    /// This is a strict check, it doesn't allow array elements or
    /// user defined properties of string value.
    fn require_string_variable(&self, idx: usize) -> Result<(), QErrorNode>;

    /// Demands that the argument at the given index is a ref value
    /// that can be cast into a string. This allows for string variables,
    /// array elements, properties of user defined types.
    fn require_string_ref(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_variable_of_built_in_type(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_variable(&self, idx: usize) -> Result<(), QErrorNode>;

    fn require_one_argument(&self) -> Result<(), QErrorNode>;

    fn require_zero_arguments(&self) -> Result<(), QErrorNode>;

    fn require_one_double_argument(&self) -> Result<(), QErrorNode> {
        self.require_one_argument()
            .and_then(|_| self.require_double_argument(0))
    }

    fn require_one_numeric_argument(&self) -> Result<(), QErrorNode> {
        self.require_one_argument()
            .and_then(|_| self.require_numeric_argument(0))
    }

    fn require_one_string_argument(&self) -> Result<(), QErrorNode> {
        self.require_one_argument()
            .and_then(|_| self.require_string_argument(0))
    }

    fn require_one_variable(&self) -> Result<(), QErrorNode> {
        self.require_one_argument()
            .and_then(|_| self.require_variable(0))
    }

    fn expr(&self, idx: usize) -> &Expression {
        self.expr_node(idx).as_ref()
    }

    fn expr_node(&self, idx: usize) -> &ExpressionNode;

    fn require_predicate<F>(&self, idx: usize, predicate: F) -> Result<(), QErrorNode>
    where
        F: Fn(&Expression) -> bool,
    {
        if predicate(self.expr(idx)) {
            Ok(())
        } else {
            Err(QError::ArgumentTypeMismatch).with_err_at(self.expr_node(idx))
        }
    }
}

impl ArgValidation for ExpressionNodes {
    fn require_integer_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        self.require_predicate(idx, |expr| expr.can_cast_to(TypeQualifier::PercentInteger))
    }

    fn require_long_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        self.require_predicate(idx, |expr| expr.can_cast_to(TypeQualifier::AmpersandLong))
    }

    fn require_double_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        self.require_predicate(idx, |expr| expr.can_cast_to(TypeQualifier::HashDouble))
    }

    fn require_numeric_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        self.require_predicate(idx, |expr| {
            if let ExpressionType::BuiltIn(q) = expr.expression_type() {
                q != TypeQualifier::DollarString
            } else {
                false
            }
        })
    }

    fn require_string_argument(&self, idx: usize) -> Result<(), QErrorNode> {
        self.require_predicate(idx, |expr| expr.can_cast_to(TypeQualifier::DollarString))
    }

    fn require_string_variable(&self, idx: usize) -> Result<(), QErrorNode> {
        match self.expr(idx) {
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

    fn require_string_ref(&self, idx: usize) -> Result<(), QErrorNode> {
        match self.expr(idx) {
            Expression::Variable(
                _,
                VariableInfo {
                    expression_type, ..
                },
            )
            | Expression::ArrayElement(
                _,
                _,
                VariableInfo {
                    expression_type, ..
                },
            )
            | Expression::Property(_, _, expression_type) => {
                if expression_type.can_cast_to(TypeQualifier::DollarString) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx])
                }
            }
            _ => Err(QError::VariableRequired).with_err_at(&self[idx]),
        }
    }

    fn require_variable_of_built_in_type(&self, idx: usize) -> Result<(), QErrorNode> {
        match self.expr(idx) {
            Expression::Variable(
                _,
                VariableInfo {
                    expression_type, ..
                },
            )
            | Expression::ArrayElement(
                _,
                _,
                VariableInfo {
                    expression_type, ..
                },
            )
            | Expression::Property(_, _, expression_type) => match expression_type {
                ExpressionType::BuiltIn(_) | ExpressionType::FixedLengthString(_) => Ok(()),
                _ => Err(QError::ArgumentTypeMismatch).with_err_at(&self[idx]),
            },
            _ => Err(QError::VariableRequired).with_err_at(&self[idx]),
        }
    }

    fn require_variable(&self, idx: usize) -> Result<(), QErrorNode> {
        if self.expr(idx).is_by_ref() {
            Ok(())
        } else {
            Err(QError::VariableRequired).with_err_at(&self[idx])
        }
    }

    fn require_one_argument(&self) -> Result<(), QErrorNode> {
        if self.len() != 1 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            Ok(())
        }
    }

    fn require_zero_arguments(&self) -> Result<(), QErrorNode> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }

    fn expr_node(&self, idx: usize) -> &ExpressionNode {
        self.get(idx).unwrap()
    }
}
