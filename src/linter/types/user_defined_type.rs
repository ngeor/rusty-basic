use super::{HasTypeDefinition, TypeDefinition};
use crate::common::{QError, StringUtils};
use crate::parser::{BareName, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, Variant};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedType {
    /// The elements
    elements: HashMap<BareName, ElementType>,
}

pub type UserDefinedTypes = HashMap<BareName, UserDefinedType>;

impl UserDefinedType {
    pub fn new(elements: HashMap<BareName, ElementType>) -> Self {
        Self { elements }
    }

    pub fn find_element(&self, element_name: &BareName) -> Option<&ElementType> {
        self.elements.get(element_name)
    }

    pub fn elements(&self) -> std::collections::hash_map::Iter<BareName, ElementType> {
        self.elements.iter()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ElementType {
    Integer,
    Long,
    Single,
    Double,
    FixedLengthString(u16),
    UserDefined(BareName),
}

impl TryFrom<&ElementType> for TypeQualifier {
    type Error = QError;

    fn try_from(value: &ElementType) -> Result<Self, Self::Error> {
        match value {
            ElementType::Integer => Ok(Self::PercentInteger),
            ElementType::Long => Ok(Self::AmpersandLong),
            ElementType::Single => Ok(Self::BangSingle),
            ElementType::Double => Ok(Self::HashDouble),
            ElementType::FixedLengthString(_) => Ok(Self::DollarString),
            _ => Err(QError::TypeMismatch),
        }
    }
}

impl ElementType {
    pub fn default_variant(&self, types: &UserDefinedTypes) -> Variant {
        match self {
            Self::Single => Variant::from(TypeQualifier::BangSingle),
            Self::Double => Variant::from(TypeQualifier::HashDouble),
            Self::FixedLengthString(len) => String::new().fix_length(*len as usize).into(),
            Self::Integer => Variant::from(TypeQualifier::PercentInteger),
            Self::Long => Variant::from(TypeQualifier::AmpersandLong),
            Self::UserDefined(type_name) => {
                Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(type_name, types)))
            }
        }
    }
}

impl HasTypeDefinition for ElementType {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::Integer => TypeDefinition::BuiltIn(TypeQualifier::PercentInteger),
            Self::Long => TypeDefinition::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Single => TypeDefinition::BuiltIn(TypeQualifier::BangSingle),
            Self::Double => TypeDefinition::BuiltIn(TypeQualifier::HashDouble),
            Self::FixedLengthString(l) => TypeDefinition::String(*l),
            Self::UserDefined(type_name) => TypeDefinition::UserDefined(type_name.clone()),
        }
    }
}
