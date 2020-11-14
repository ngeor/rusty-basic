use crate::common::StringUtils;
use crate::linter::{ArrayDimensions, ExpressionType, HasExpressionType, UserDefinedTypes};
use crate::parser::{BareName, BuiltInStyle, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, Variant};

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    // A -> A!
    // A AS STRING
    // A$, A% etc
    BuiltIn(TypeQualifier, BuiltInStyle),

    // DIM C AS Card
    UserDefined(BareName),

    /// DIM X AS STRING * 1
    FixedLengthString(u16),

    Array(ArrayDimensions, Box<ExpressionType>),
}

impl DimType {
    pub fn default_variant(&self, types: &UserDefinedTypes) -> Variant {
        match self {
            Self::BuiltIn(q, _) => Variant::from(*q),
            Self::FixedLengthString(len) => String::new().fix_length(*len as usize).into(),
            Self::UserDefined(type_name) => {
                Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(type_name, types)))
            }
            _ => unimplemented!(),
        }
    }
}

impl HasExpressionType for DimType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier, _) => ExpressionType::BuiltIn(*qualifier),
            Self::FixedLengthString(len) => ExpressionType::FixedLengthString(*len),
            Self::UserDefined(type_name) => ExpressionType::UserDefined(type_name.clone()),
            Self::Array(_, element_type) => ExpressionType::Array(element_type.clone()),
            Self::Bare => panic!("Unresolved type"),
        }
    }
}
