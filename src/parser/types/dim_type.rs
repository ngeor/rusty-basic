use crate::parser::{ArrayDimensions, BareNameNode, BuiltInStyle, ExpressionNode, TypeQualifier};

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    FixedLengthString(ExpressionNode),
    UserDefined(BareNameNode),
    Array(ArrayDimensions, Box<DimType>),
}
