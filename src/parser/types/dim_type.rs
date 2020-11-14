use crate::parser::{ArrayDimensions, BareNameNode, BuiltInStyle, ExpressionNode, TypeQualifier};

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    FixedLengthString(ExpressionNode, u16),
    UserDefined(BareNameNode),
    Array(ArrayDimensions, Box<DimType>),
}
