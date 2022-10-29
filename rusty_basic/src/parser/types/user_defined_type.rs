use std::collections::HashMap;
use std::slice::Iter;

use crate::parser::types::{
    BareName, BareNameNode, ExpressionNode, ExpressionType, HasExpressionType, Name, TypeQualifier,
};
use crate::variant::{UserDefinedTypeValue, Variant};
use rusty_common::{Locatable, QError, StringUtils};

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedType {
    /// The name of the type
    name: BareName,
    /// Comments between the type name and the first element
    comments: Vec<Locatable<String>>,
    /// The elements
    elements: Vec<ElementNode>,
}

pub type UserDefinedTypes = HashMap<BareName, UserDefinedType>;

impl UserDefinedType {
    pub fn new(
        name: BareName,
        comments: Vec<Locatable<String>>,
        elements: Vec<ElementNode>,
    ) -> Self {
        Self {
            name,
            comments,
            elements,
        }
    }

    pub fn bare_name(&self) -> &BareName {
        &self.name
    }

    pub fn elements(&self) -> Iter<'_, ElementNode> {
        self.elements.iter()
    }

    fn find_element_type(&self, element_name: &BareName) -> Option<&ElementType> {
        self.elements
            .iter()
            .map(|Locatable { element, .. }| element)
            .find(|x| &x.name == element_name)
            .map(|x| &x.element_type)
    }

    pub fn demand_element_by_name(&self, element_name: &Name) -> Result<&ElementType, QError> {
        let element_type = self
            .find_element_type(element_name.bare_name())
            .ok_or(QError::ElementNotDefined)?;
        if element_type.can_be_referenced_by_property_name(element_name) {
            Ok(element_type)
        } else {
            Err(QError::TypeMismatch)
        }
    }
}

pub type ElementNode = Locatable<Element>;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// The name of the element
    pub name: BareName,
    /// The element type
    pub element_type: ElementType,
    /// Comments between the end of this element and the next one
    pub comments: Vec<Locatable<String>>,
}

impl Element {
    pub fn new(
        name: BareName,
        element_type: ElementType,
        comments: Vec<Locatable<String>>,
    ) -> Self {
        Self {
            name,
            element_type,
            comments,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Integer,
    Long,
    Single,
    Double,
    FixedLengthString(ExpressionNode, u16),
    UserDefined(BareNameNode),
}

impl ElementType {
    pub fn default_variant(&self, types: &UserDefinedTypes) -> Variant {
        match self {
            Self::Single => Variant::from(TypeQualifier::BangSingle),
            Self::Double => Variant::from(TypeQualifier::HashDouble),
            Self::FixedLengthString(_, len) => Variant::VString("".fix_length(*len as usize)),
            Self::Integer => Variant::from(TypeQualifier::PercentInteger),
            Self::Long => Variant::from(TypeQualifier::AmpersandLong),
            Self::UserDefined(Locatable { element, .. }) => {
                Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(element, types)))
            }
        }
    }

    pub fn can_be_referenced_by_property_name(&self, name: &Name) -> bool {
        match name {
            Name::Bare(_) => true,
            Name::Qualified(_, qualifier) => match self {
                Self::Integer => *qualifier == TypeQualifier::PercentInteger,
                Self::Long => *qualifier == TypeQualifier::AmpersandLong,
                Self::Single => *qualifier == TypeQualifier::BangSingle,
                Self::Double => *qualifier == TypeQualifier::HashDouble,
                Self::FixedLengthString(_, _) => *qualifier == TypeQualifier::DollarString,
                _ => false,
            },
        }
    }
}

impl HasExpressionType for ElementType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Integer => ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
            Self::Long => ExpressionType::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Single => ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            Self::Double => ExpressionType::BuiltIn(TypeQualifier::HashDouble),
            Self::FixedLengthString(_, l) => ExpressionType::FixedLengthString(*l),
            Self::UserDefined(Locatable { element, .. }) => {
                ExpressionType::UserDefined(element.clone())
            }
        }
    }
}
