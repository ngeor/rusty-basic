use crate::linter::{ArrayDimensions, ExpressionNode, ExpressionType, HasExpressionType};
use crate::parser::{BareNameNode, BuiltInStyle, TypeQualifier};

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    // A -> A!
    // A AS STRING
    // A$, A% etc
    BuiltIn(TypeQualifier, BuiltInStyle),

    // DIM C AS Card
    UserDefined(BareNameNode),

    /// DIM X AS STRING * 1
    FixedLengthString(ExpressionNode, u16),

    Array(ArrayDimensions, Box<DimType>),
}

impl HasExpressionType for DimType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier, _) => ExpressionType::BuiltIn(*qualifier),
            Self::FixedLengthString(_, len) => ExpressionType::FixedLengthString(*len),
            Self::UserDefined(type_name) => ExpressionType::UserDefined(type_name.element.clone()),
            Self::Array(_, element_type) => {
                ExpressionType::Array(Box::new(element_type.expression_type()))
            }
            Self::Bare => panic!("Unresolved type"),
        }
    }
}
