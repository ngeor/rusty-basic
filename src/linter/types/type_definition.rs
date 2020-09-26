use super::UserDefinedTypes;
use crate::common::{CanCastTo, StringUtils};
use crate::parser::{BareName, Operator, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, Variant};

// TODO is it possible to get rid of TypeDefinition?

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

    pub fn cast_binary_op(&self, right: TypeDefinition, op: Operator) -> Option<TypeDefinition> {
        match self {
            TypeDefinition::BuiltIn(q_left) => match right {
                TypeDefinition::BuiltIn(q_right) => q_left
                    .cast_binary_op(q_right, op)
                    .map(|q_result| TypeDefinition::BuiltIn(q_result)),
                TypeDefinition::String(_) => q_left
                    .cast_binary_op(TypeQualifier::DollarString, op)
                    .map(|q_result| TypeDefinition::BuiltIn(q_result)),
                _ => None,
            },
            TypeDefinition::String(_) => match right {
                TypeDefinition::BuiltIn(TypeQualifier::DollarString)
                | TypeDefinition::String(_) => TypeQualifier::DollarString
                    .cast_binary_op(TypeQualifier::DollarString, op)
                    .map(|q_result| TypeDefinition::BuiltIn(q_result)),
                _ => None,
            },
            _ => None,
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
