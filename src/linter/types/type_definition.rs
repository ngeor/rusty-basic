use super::UserDefinedTypes;
use crate::common::{CanCastTo, StringUtils};
use crate::parser::{BareName, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, Variant};

/// A linted (resolved) `TypeDefinition`.
///
/// Similar to the one defined in `parser` but without `Bare` and with `FileHandle`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeDefinition {
    BuiltIn(TypeQualifier),
    String(u16),
    UserDefined(BareName),
    FileHandle,
}

impl TypeDefinition {
    pub fn default_variant(&self, types: &UserDefinedTypes) -> Variant {
        match self {
            Self::BuiltIn(q) => Variant::from(*q),
            Self::String(len) => String::new().fix_length(*len as usize).into(),
            Self::UserDefined(type_name) => {
                Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(type_name, types)))
            }
            Self::FileHandle => panic!("not possible to get a default file handle"),
        }
    }
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
