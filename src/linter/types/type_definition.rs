use crate::common::CanCastTo;
use crate::parser::{BareName, TypeQualifier};

/// A linted (resolved) `TypeDefinition`.
///
/// Similar to the one defined in `parser` but without `Bare` and with `FileHandle`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeDefinition {
    BuiltIn(TypeQualifier),
    String(u32),
    UserDefined(BareName),
    FileHandle,
}

impl CanCastTo<&TypeDefinition> for TypeDefinition {
    fn can_cast_to(&self, other: &Self) -> bool {
        match self {
            Self::BuiltIn(q_left) => match other {
                Self::BuiltIn(q_right) => q_left.can_cast_to(*q_right),
                Self::String(_) => *q_left == TypeQualifier::DollarString,
                _ => false,
            },
            Self::String(_) => match other {
                Self::BuiltIn(TypeQualifier::DollarString) | Self::String(_) => true,
                _ => false,
            },
            Self::UserDefined(u_left) => match other {
                Self::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
            Self::FileHandle => false,
        }
    }
}

impl CanCastTo<TypeQualifier> for TypeDefinition {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        match self {
            Self::BuiltIn(q_left) => q_left.can_cast_to(other),
            Self::String(_) => other == TypeQualifier::DollarString,
            _ => false,
        }
    }
}
