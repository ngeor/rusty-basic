use crate::parser::{ExpressionType, TypeQualifier};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnaryOperator {
    // Plus,
    Minus,
    Not,
}
