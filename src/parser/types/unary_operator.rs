use crate::parser::{ExpressionType, TypeQualifier};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnaryOperator {
    // Plus,
    Minus,
    Not,
}

impl UnaryOperator {
    pub fn applies_to(&self, expr_type: &ExpressionType) -> bool {
        match expr_type {
            ExpressionType::BuiltIn(TypeQualifier::BangSingle)
            | ExpressionType::BuiltIn(TypeQualifier::HashDouble)
            | ExpressionType::BuiltIn(TypeQualifier::PercentInteger)
            | ExpressionType::BuiltIn(TypeQualifier::AmpersandLong) => true,
            _ => false,
        }
    }
}
