use super::{BareName, Name, TypeQualifier};

/// Defines a variable using a DIM statement
#[derive(Clone, Debug, PartialEq)]
pub enum DimDefinition {
    /// The DIM statement does not include an AS clause, the type is derived by the name
    Compact(Name),
    /// The DIM statement has an AS clause specifying the type
    Extended(BareName, DimType),
}

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    /// A built-in type designated in code by one of the keywords INTEGER, LONG, SINGLE, DOUBLE or STRING.
    BuiltInType(TypeQualifier),

    /// A user defined type.
    UserDefinedType(BareName),
}

impl From<TypeQualifier> for DimType {
    fn from(q: TypeQualifier) -> Self {
        Self::BuiltInType(q)
    }
}
